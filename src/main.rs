#[macro_use]
extern crate lazy_static;

use chrono::Utc;
use observability::Metering;
use opentelemetry::global::shutdown_tracer_provider;
use tracing::{debug, info, span, trace, warn};

use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use settings::Settings;
use simulation::Simulation;
use tokio::time::{sleep, Duration, Instant};

use crate::{observability::init_tracing, simulation::SimulationParameters};

mod device;
mod generator;
mod observability;
mod settings;
mod simulation;

lazy_static! {
    static ref CONFIG: Settings = Settings::new();
}

#[tracing::instrument]
#[tokio::main]
async fn main() {
    init_tracing();

    let (client, eventloop) = connect_broker().await;
    wait_for_start_time().await;

    let params = get_parameters();
    let simulation_handle = tokio::spawn(async move { simulate(client, params).await });
    let listen_handle = tokio::spawn(async move { listen(eventloop).await });
    futures::future::select(simulation_handle, listen_handle).await;

    info!("Shutting down.");
    shutdown_tracer_provider();
}

async fn connect_broker() -> (AsyncClient, EventLoop) {
    info!(broker_url = &CONFIG.broker_url,
        broker_user = &CONFIG.broker_user, broker_pass = anonymize(&CONFIG.broker_pass),
        broker_client_id = &CONFIG.broker_client_id, broker_qos = CONFIG.broker_qos,
        otlp_collector = ?CONFIG.otlp_collector, otlp_auth = anonymize_opt(&CONFIG.otlp_auth),
        capacity = CONFIG.capacity, sim_start_time = ?CONFIG.sim_start_time,
        "Connecting to broker.");
    create_mqtt_client().await
}

async fn wait_for_start_time() {
    if let Some(start_time) = CONFIG.sim_start_time {
        let now = Utc::now();
        let wait_time = (start_time - now).num_milliseconds().max(0) as u64;
        sleep(Duration::from_millis(wait_time)).await;
    }
}

fn get_parameters() -> SimulationParameters {
    info!(
        sim_devices = CONFIG.sim_devices,
        sim_data_points = CONFIG.sim_data_points,
        sim_seed = CONFIG.sim_seed,
        sim_frequency_secs = CONFIG.sim_frequency_secs,
        sim_runs = CONFIG.sim_runs,
        "Running the simulation."
    );
    SimulationParameters {
        client_id: CONFIG.broker_client_id.clone(),
        devices: CONFIG.sim_devices,
        data_points: CONFIG.sim_data_points,
        seed: CONFIG.sim_seed,
        frequency_secs: CONFIG.sim_frequency_secs,
        qos: CONFIG.broker_qos,
    }
}

async fn simulate(client: AsyncClient, parms: SimulationParameters) {
    let metering = Metering::new();

    let mut simulation = Simulation::new(&parms);
    let frequency = Duration::from_secs(parms.frequency_secs);
    let datapoints = parms.devices * parms.data_points;
    let qos = get_qos();

    for _ in 0..CONFIG.sim_runs {
        let simulation_span = span!(tracing::Level::INFO, "simulation_run");
        let _enter = simulation_span.enter();
        debug!(parent: &simulation_span, sim_devices = parms.devices, sim_data_points = parms.data_points, sim_frequency = parms.frequency_secs, sim_seed = parms.seed, "Running simulation");

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
        let remainder = frequency.saturating_sub(elapsed);
        if remainder == Duration::ZERO {
            metering.is_overloaded();
            warn!(parent: &simulation_span, "Messages cannot be sent fast enough. Increase capacity on receiving end, increase wait time or reduce the number of data points.");
        }
        metering.record_datapoints(datapoints, frequency);
        metering.record_capacity(elapsed, frequency);
        debug!(parent: &simulation_span, remainder=?remainder, "Sleeping");
        sleep(remainder).await;
    }
}

fn get_qos() -> QoS {
    match CONFIG.broker_qos {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => panic!("Invalid QoS level."),
    }
}

/// Listen for incoming messages and handle them. If I don't handle the incoming messages, sending messages will block.
async fn listen(mut eventloop: EventLoop) {
    loop {
        match eventloop.poll().await {
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

/// Create the MQTT connection based on the configuration.
async fn create_mqtt_client() -> (AsyncClient, EventLoop) {
    let url = format!(
        "{}?client_id={}",
        CONFIG.broker_url, CONFIG.broker_client_id
    );
    let mut opts = MqttOptions::parse_url(url).unwrap();

    opts.set_credentials(&CONFIG.broker_user, &CONFIG.broker_pass);
    opts.set_keep_alive(Duration::from_secs(5));

    AsyncClient::new(opts, CONFIG.capacity)
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
