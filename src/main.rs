#[macro_use]
extern crate lazy_static;
extern crate log;

use std::env;

use log::{debug, info, warn};
use observability::Metering;
use tracing::span;

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

/// Main loop of running the simulation and receiving commands to control the simulation through MQTT.
#[tokio::main]
async fn main() {
    // Should be probably either log module or tracing.
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init();
    init_tracing();

    let (client, eventloop) = create_mqtt_client().await;
    let (params_tx, params_rx) = watch::channel(SimulationParameters::default());
    let simulation_handle = tokio::spawn(async move { simulate(client, params_rx).await });
    let command_handle = tokio::spawn(async move { listen(eventloop, params_tx).await });
    info!("Started, waiting for commands...\n{:?}", *CONFIG);
    futures::future::select(simulation_handle, command_handle).await;
    print!("Exiting...");
    /*
        TODO: Handle the situation when there is a connection error somewhere.
        The simulation should try to reconnect and continue with the same simulation parameters,
        but for that it would need to preserve the parameters.
    */
}

async fn listen(mut eventloop: EventLoop, params_tx: watch::Sender<SimulationParameters>) {
    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(msg))) => {
                debug!("Received incoming publish: {:?}", msg);
                if let Err(e) = handle_cmd(msg.payload.to_vec(), &params_tx) {
                    warn!("Send error: {:?}", e);
                    return;
                }
            }
            Ok(Event::Incoming(Packet::Disconnect)) => {
                warn!("Disconnected from the broker.");
                return;
            }
            Ok(_) => {
                //debug!("Received: {:?}", x);
            }
            Err(e) => {
                warn!("Failed to connect: {}", e);
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
                simulation.start(params);
            } else {
                simulation.stop();
                remainder = params.wait_time.clone();
                // For whatever reason: If I don't wait at least a little bit here, this runs into an infinite loop if the other end is not connected right from the start.
                sleep(Duration::from_millis(1)).await;
                continue;
            }
        }

        let simulation_span = span!(tracing::Level::INFO, "simulation_run");
        let _enter = simulation_span.enter();
        tracing::info!(parent: &simulation_span, "Running simulation for {:?}", params);

        let start = Instant::now();
        info!("Running simulation for {:?}", params);
        for (topic, data) in simulation.iter() {
            match client.publish(topic, qos, false, data).await {
                Ok(_) => (),
                Err(e) => {
                    warn!("Failed to publish: {}", e);
                    return;
                }
            }
        }

        let elapsed = start.elapsed();
        remainder = params.wait_time.saturating_sub(elapsed);
        info!("Sleeping for {:?}", remainder);
        if remainder == Duration::ZERO {
            metering.is_overloaded();
            warn!("Messages cannot be sent fast enough. Increase capacity on receiving end, increase wait time or reduce the number of data points.");
        }
        metering.record_datapoints(datapoints, params.wait_time);
        metering.record_capacity(elapsed, params.wait_time);
        tracing::info!(parent: &simulation_span, "Sleeping for {:?}", remainder);
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
