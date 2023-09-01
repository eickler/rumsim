use log::debug;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::sync::mpsc;
use std::time::Duration;

use crate::{
    generator::{create, Generator, GeneratorType},
    settings::Target,
};

pub struct Device {
    thread_id: u32,
    generators: Vec<Box<dyn Generator>>,
    wait_time_secs: u16,
    receiver: mpsc::Receiver<bool>,
    mqtt: AsyncClient,
}

impl Device {
    pub fn new(thread_id: u32, receiver: mpsc::Receiver<bool>, target: Target) -> Self {
        let seed = target.seed;
        let data_points = target.data_points;
        let generators = create_data_point_generators(seed, data_points);

        let mqtt = create_mqtt_connection(&thread_id, &target);

        let wait_time_secs = target.wait_time_secs;
        Device {
            thread_id,
            generators,
            wait_time_secs,
            receiver,
            mqtt,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.work();
            if self.nap_or_stop() {
                break;
            }
        }
        debug!("Thread {} stopping", self.thread_id);
    }

    fn work(&mut self) {
        debug!("Thread {} working", self.thread_id);
        // Generate a timestamp

        for generator in self.generators.iter_mut() {
            // generate a name --> Sollte der Generator machen
            let value = generator.generate();
            // value is Into<Vec<u8>>
            //mqtt.publish(topic, QoS::AtLeastOnce, false, msg);
        }
    }

    fn nap_or_stop(&mut self) -> bool {
        let nap_time = Duration::from_secs(self.wait_time_secs.into());
        if let Ok(_msg) = self.receiver.recv_timeout(nap_time) {
            return true;
        }
        false
    }
}

fn create_data_point_generators(seed: u64, data_points: u16) -> Vec<Box<dyn Generator>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut generators = Vec::new();

    for _i in 0..data_points / 3 {
        let generator = create(GeneratorType::Status, rng.gen());
        generators.push(generator);
    }

    for _i in data_points / 3..2 * data_points / 3 {
        let generator = create(GeneratorType::Noise, rng.gen());
        generators.push(generator);
    }

    for _i in 2 * data_points / 3..data_points {
        let generator = create(GeneratorType::Sensor, rng.gen());
        generators.push(generator);
    }
    generators
}

fn create_mqtt_connection(thread_id: &u32, target: &Target) -> AsyncClient {
    let cap = target.data_points.into(); // TODO: Is this a good idea? Background is that we send all data points out for delivery at once.
    let thread_str = thread_id.to_string();
    let url = target.url.clone() + "?client_id=device_" + thread_str.as_str();
    let mut opts = MqttOptions::parse_url(url).unwrap();
    opts.set_credentials(&target.user, &target.pass);
    opts.set_keep_alive(Duration::from_secs(5));

    let (client, mut _eventloop) = AsyncClient::new(opts, cap);
    client
}
