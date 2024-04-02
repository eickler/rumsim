#[macro_use]
extern crate lazy_static;

use observability::Metering;
use opentelemetry::global::shutdown_tracer_provider;
use tracing::{debug, info, span, trace, warn};

use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use settings::Settings;
use simulation::{Simulation, SimulationParameters};
use tokio::sync::watch;
use tokio::time::{sleep, timeout, Duration, Instant};

use crate::commands::Command::{Start, Stop};
use crate::observability::init_tracing;

mod commands;
mod device;
mod generator;
mod observability;
mod settings;
mod simulation;

lazy_static! {
    static ref CONFIG: Settings = Settings::new();
}

fn anonymize(s: &str) -> String {
    format!("{}…{}", &s[..1], &s[s.len() - 1..])
}

fn anonymize_opt(s: &Option<String>) -> String {
    match s {
        Some(s) => anonymize(s),
        None => "None".to_string(),
    }
}

#[tracing::instrument]
#[tokio::main]
async fn main() {
    init_tracing();

    let (client, eventloop) = create_mqtt_client().await;
    let (params_tx, params_rx) = watch::channel(SimulationParameters::default());
    let simulation_handle = tokio::spawn(async move { simulate(client, params_rx).await });
    let command_handle = tokio::spawn(async move { listen(eventloop, params_tx).await });

    info!(mqtt_url = &CONFIG.url, mqtt_client_id = &CONFIG.client_id,
        mqtt_user = &CONFIG.user, mqtt_pass = anonymize(&CONFIG.pass), mqtt_qos = CONFIG.qos,
        otlp_collector = ?CONFIG.otlp_collector, otlp_auth = anonymize_opt(&CONFIG.otlp_auth),
        "Started, waiting for commands...");
    futures::future::select(simulation_handle, command_handle).await;

    info!("Shutting down...");
    shutdown_tracer_provider();
}

async fn listen(mut eventloop: EventLoop, params_tx: watch::Sender<SimulationParameters>) {
    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(msg))) => {
                trace!(message = ?msg, "Received publish");
                if let Err(e) = handle_cmd(msg.payload.to_vec(), &params_tx) {
                    warn!(error = ?e, "Send error");
                    return;
                }
            }
            Ok(Event::Incoming(Packet::Disconnect)) => {
                warn!("Disconnected from the broker.");
                return;
            }
            Ok(x) => {
                trace!(message = ?x, "Received message");
            }
            Err(e) => {
                warn!(error = ?e, "Failed to connect");
                return;
            }
        }
    }
}

fn handle_cmd(
    payload: Vec<u8>,
    params_tx: &watch::Sender<SimulationParameters>,
) -> Result<(), watch::error::SendError<SimulationParameters>> {
    if let Ok(command_str) = String::from_utf8(payload) {
        let command = commands::parse(&command_str);
        return match command {
            Ok(Start(new_params)) => params_tx.send(new_params),
            Ok(Stop) => params_tx.send(SimulationParameters::default()),
            _ => Ok({
                warn!("Invalid command: {}", command_str);
                ()
            }),
        };
    }
    Ok(())
}

async fn simulate(client: AsyncClient, mut params_rx: watch::Receiver<SimulationParameters>) {
    let metering = Metering::new();

    let qos = get_qos();
    let mut simulation = Simulation::new(&CONFIG.client_id);
    let mut params = SimulationParameters::default();
    let mut datapoints = params.devices * params.data_points;
    let mut remainder = params.wait_time.clone();

    loop {
        if let Ok(_) = timeout(remainder, params_rx.changed()).await {
            params = params_rx.borrow_and_update().clone();
            datapoints = params.devices * params.data_points;
            if params.devices > 0 {
                info!(devices = params.devices, data_points = params.data_points, wait_time = ?params.wait_time, seed = params.seed, "Starting simulation");
                simulation.start(params);
            } else {
                info!("Stopping simulation.");
                simulation.stop();
                remainder = params.wait_time.clone();
                // For whatever reason: If I don't wait at least a little bit here, this runs into an infinite loop if the other end is not connected right from the start.
                sleep(Duration::from_millis(1)).await;
                continue;
            }
        }

        let simulation_span = span!(tracing::Level::INFO, "simulation_run");
        let _enter = simulation_span.enter();
        debug!(parent: &simulation_span, devices = params.devices, data_points = params.data_points, wait_time = ?params.wait_time, seed = params.seed, "Running simulation");

        let start = Instant::now();
        for (topic, data) in simulation.iter() {
            match client.publish(topic, qos, false, data).await {
                Ok(_) => (),
                Err(e) => {
                    warn!(error = ?e, "Failed to publish");
                    return;
                }
            }
        }

        let elapsed = start.elapsed();
        remainder = params.wait_time.saturating_sub(elapsed);
        if remainder == Duration::ZERO {
            metering.is_overloaded();
            warn!(parent: &simulation_span, "Messages cannot be sent fast enough. Increase capacity on receiving end, increase wait time or reduce the number of data points.");
        }
        metering.record_datapoints(datapoints, params.wait_time);
        metering.record_capacity(elapsed, params.wait_time);
        debug!(parent: &simulation_span, remainder=?remainder, "Sleeping");
    }
}

fn get_qos() -> QoS {
    match CONFIG.qos {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => panic!("Invalid QoS level."),
    }
}

/// Create the MQTT connection based on the configuration.
async fn create_mqtt_client() -> (AsyncClient, EventLoop) {
    let url = format!("{}?client_id={}", CONFIG.url, CONFIG.client_id);
    let mut opts = MqttOptions::parse_url(url).unwrap();

    opts.set_credentials(&CONFIG.user, &CONFIG.pass);
    opts.set_keep_alive(Duration::from_secs(5));

    let (client, eventloop) = AsyncClient::new(opts, CONFIG.capacity);
    client
        .subscribe(&CONFIG.control_topic, QoS::AtMostOnce)
        .await
        .unwrap(); // It's OK if this panics when there's no connection.

    (client, eventloop)
}
