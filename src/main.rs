#[macro_use]
extern crate lazy_static;
extern crate log;

use rumqttc::Event::Incoming;
use rumqttc::Packet::Publish;
use rumqttc::{AsyncClient, ConnectionError, MqttOptions, QoS};

use std::time::Duration;

mod commands;
mod control;
mod device;
mod generator;
mod settings;

lazy_static! {
    static ref CONFIG: settings::Settings =
        settings::Settings::new().expect("Configuration cannot be loaded.");
}

#[tokio::main]
async fn main() {
    env_logger::init();

    /*
     * Client starts and starts listening for commands on a "control plane" broker.
     * Receives start command with amount of devices, amount of data points and milliseconds wait.
     * Start stops everything running and start per device a thread with a wait of milliseconds/devices ms.
     * Threads produce every wait interval the amount of data points for their device.
     */

    let config = &CONFIG.control;
    let url = config.url.clone() + "?client_id=123";
    let mut opts = MqttOptions::parse_url(url).unwrap();
    opts.set_credentials(&config.user, &config.pass);
    opts.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(opts, 10);
    client
        .subscribe(&config.topic, QoS::AtMostOnce)
        .await
        .unwrap();

    let mut control = control::Control::new(&CONFIG.target);

    loop {
        match eventloop.poll().await {
            Ok(Incoming(Publish(msg))) => {
                println!("Received incoming publish {:?}", msg);
                if let Ok(command_str) = String::from_utf8(msg.payload.to_vec()) {
                    let command = commands::parse(&command_str);
                    match command {
                        Ok(cmd) => control.run(&cmd),
                        _ => println!("Invalid command: {}", command_str),
                    }
                }
            }
            Ok(x) => {
                println!("Received = {:?}", x);
            }
            Err(e) => {
                eprintln!("Failed to connect: {}", e);
            }
        }
    }
}
