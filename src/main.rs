#[macro_use]
extern crate lazy_static;
extern crate log;

use crate::commands::Command::{Start, Stop};
use log::{info, warn};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use simulation::{Simulation, SimulationParameters};
use tokio::sync::watch;
use tokio::time::{Duration, Instant};

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
    let (client, eventloop) = create_mqtt_client().await;
    let (params_tx, params_rx) = watch::channel(SimulationParameters::default());
    let simulation_handle = tokio::spawn(async move { simulate(client, params_rx).await });
    let command_handle = tokio::spawn(async move { listen(eventloop, params_tx).await });
    futures::future::select(simulation_handle, command_handle).await;
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
                info!("Received incoming publish: {:?}", msg);
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
    let mut simulation = Simulation::new();
    let mut params = SimulationParameters::default();

    loop {
        if let Ok(changed) = params_rx.has_changed() {
            if changed {
                params = params_rx.borrow_and_update().clone();
                info!("Parameters changed: {:?}", params);
                if params.devices > 0 {
                    simulation.start(params);
                } else {
                    simulation.stop();
                }
            }
        }

        let start = Instant::now();
        info!("Running simulation for {:?}", params);
        for (topic, data) in simulation.iter() {
            match client.publish(topic, QoS::AtLeastOnce, false, data).await {
                Ok(_) => (),
                Err(e) => {
                    warn!("Failed to publish: {}", e);
                    return;
                }
            }
        }
        let remainder = params.wait_time.saturating_sub(start.elapsed());
        info!("Sleeping for {:?}", remainder);
        tokio::time::sleep(remainder).await;
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
