#[macro_use]
extern crate lazy_static;

use observability::Metering;
use opentelemetry::global::shutdown_tracer_provider;
use tracing::{debug, info, span, trace, warn};

use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use settings::Settings;
use simulation::Simulation;
use tokio::time::{sleep, Duration, Instant};

use crate::observability::init_tracing;

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

    info!(broker_url = &CONFIG.broker_url,
        broker_user = &CONFIG.broker_user, broker_pass = anonymize(&CONFIG.broker_pass),
        broker_client_id = &CONFIG.broker_client_id, broker_qos = CONFIG.broker_qos,
        otlp_collector = ?CONFIG.otlp_collector, otlp_auth = anonymize_opt(&CONFIG.otlp_auth),
        capacity = CONFIG.capacity, sim_wait_time_secs = CONFIG.sim_wait_time_secs,
        "Connecting to broker.");
    let (client, eventloop) = create_mqtt_client().await;
    let simulation_handle = tokio::spawn(async move { simulate(client).await });
    let listen_handle = tokio::spawn(async move { listen(eventloop).await });

    futures::future::select(simulation_handle, listen_handle).await;

    info!("Shutting down.");
    shutdown_tracer_provider();
}

async fn simulate(client: AsyncClient) {
    sleep(Duration::from_secs(CONFIG.sim_wait_time_secs)).await;
    let metering = Metering::new();

    info!(
        sim_devices = CONFIG.sim_devices,
        sim_data_points = CONFIG.sim_data_points,
        sim_seed = CONFIG.sim_seed,
        sim_frequency_secs = CONFIG.sim_frequency_secs,
        sim_runs = CONFIG.sim_runs,
        "Running the simulation."
    );

    let mut simulation = Simulation::new(
        &CONFIG.broker_client_id,
        CONFIG.sim_devices,
        CONFIG.sim_data_points,
        CONFIG.sim_seed,
    );

    let frequency = Duration::from_secs(CONFIG.sim_frequency_secs);
    let datapoints = CONFIG.sim_devices * CONFIG.sim_data_points;
    let qos = get_qos();

    // TBD: implement the runs
    loop {
        let simulation_span = span!(tracing::Level::INFO, "simulation_run");
        let _enter = simulation_span.enter();
        debug!(parent: &simulation_span, sim_devices = CONFIG.sim_devices, sim_data_points = CONFIG.sim_data_points, sim_frequency = CONFIG.sim_frequency_secs, sim_seed = CONFIG.sim_seed, "Running simulation");

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
