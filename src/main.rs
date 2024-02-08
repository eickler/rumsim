#[macro_use]
extern crate lazy_static;
extern crate log;

use log::{debug, info, warn};
use rumqttc::{AsyncClient, ConnectionError, Event, EventLoop, MqttOptions, Packet, QoS};
use simulation::Simulation;
use tokio::time::{timeout, Duration, Instant};

mod commands;
mod device;
mod generator;
mod settings;
mod simulation;

lazy_static! {
    static ref CONFIG: settings::Settings =
        settings::Settings::new().expect("Configuration cannot be loaded.");
}

/// Main loop of running the simulation and receiving commands to control the simulation through MQTT.
#[tokio::main]
async fn main() {
    env_logger::init();

    let (client, mut eventloop) = create_mqtt_client().await;
    let mut simulation = Simulation::new();

    loop {
        let wait_time = simulate(&mut simulation, &client).await;
        wait_for_command(wait_time, &mut eventloop, &mut simulation).await;
    }
}

/// Run the simulation and return the remaining time to wait until the next simulation cycle.
async fn simulate(simulation: &mut Simulation, client: &AsyncClient) -> Duration {
    let wait_time = simulation.interval();
    let start = Instant::now();
    simulation.run(&client).await;
    wait_time.saturating_sub(start.elapsed())
}

/// Wait until the next simulation cycle, potentially receiving commands to control the simulation meanwhile.
async fn wait_for_command(
    wait_time: Duration,
    eventloop: &mut EventLoop,
    simulation: &mut Simulation,
) {
    let mut wait_time = wait_time.clone();
    loop {
        let start = Instant::now();
        debug!("Wait time {:#?}", wait_time);
        match timeout(wait_time, eventloop.poll()).await {
            Ok(message) => {
                try_configure(simulation, message);
                wait_time = wait_time.saturating_sub(start.elapsed());
            }
            _ => {
                debug!("Wait time elapsed");
                return;
            }
        }
    }
}

/// Create the MQTT connection based on the configuration.
async fn create_mqtt_client() -> (AsyncClient, EventLoop) {
    let config = &CONFIG.control;
    let url = format!("{}?client_id={}", config.url, config.client_id);
    let mut opts = MqttOptions::parse_url(url).unwrap();

    opts.set_credentials(&config.user, &config.pass);
    opts.set_keep_alive(Duration::from_secs(5));

    let (client, eventloop) = AsyncClient::new(opts, config.capacity);
    client
        .subscribe(&config.control_topic, QoS::AtMostOnce)
        .await
        .unwrap(); // It's OK if this panics when there's no connection.

    (client, eventloop)
}

/// Configure the simulation based on configuration commands sent to this client.
fn try_configure(simulation: &mut Simulation, message: Result<Event, ConnectionError>) {
    match message {
        Ok(Event::Incoming(Packet::Publish(msg))) => {
            info!("Received incoming publish {:?}", msg);
            if let Ok(command_str) = String::from_utf8(msg.payload.to_vec()) {
                let command = commands::parse(&command_str);
                match command {
                    Ok(cmd) => simulation.configure(&cmd),
                    _ => println!("Invalid command: {}", command_str),
                }
            }
        }
        Ok(Event::Incoming(Packet::Disconnect)) => {
            // TODO: If there is an error here, I probably need to rescubscribe to the command channel.
            warn!("Disconnected from the broker");
        }
        Ok(x) => {
            debug!("Received = {:?}", x);
        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
        }
    }
}
