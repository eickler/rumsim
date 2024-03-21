#[macro_use]
extern crate lazy_static;
extern crate log;

use crate::commands::Command::{Start, Stop};
use log::{debug, info, warn};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use simulation::{Simulation, SimulationParameters};
use tokio::sync::watch;
use tokio::time::{sleep, timeout, Duration, Instant};

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
    info!("Started, waiting for commands...");
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
    let qos = get_qos();
    let mut simulation = Simulation::new();
    let mut params = SimulationParameters::default();
    let mut remainder = params.wait_time.clone();

    loop {
        if let Ok(_) = timeout(remainder, params_rx.changed()).await {
            params = params_rx.borrow_and_update().clone();
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
        remainder = params.wait_time.saturating_sub(start.elapsed());
        info!("Sleeping for {:?}", remainder);
        if remainder == Duration::ZERO {
            warn!("Messages cannot be sent fast enough. Increase capacity on receiving end, increase wait time or reduce the number of data points.");
        }
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
