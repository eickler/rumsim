use async_trait::async_trait;
use futures::executor::block_on;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::{
    sync::mpsc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(not(test))]
use log::debug;
#[cfg(test)]
use mockall::automock;
#[cfg(test)]
use std::println as debug;

use crate::{
    generator::{create_generator, Generator, GeneratorType},
    settings::Target,
};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait MqttClient {
    async fn publish(&self, topic: String, payload: String);
}

#[async_trait]
impl MqttClient for rumqttc::AsyncClient {
    async fn publish(&self, topic: String, payload: String) {
        self.publish(topic, QoS::AtLeastOnce, false, payload)
            .await
            .unwrap_or_else(|e| {
                eprintln!("Failed to publish: {}", e);
            });
    }
}

pub struct Device<T: MqttClient> {
    thread_id: u32,
    name: String,
    generators: Vec<Box<dyn Generator>>,
    wait_time_secs: u16,
    receiver: mpsc::Receiver<bool>,
    mqtt: T,
}

pub fn create_device(
    thread_id: u32,
    receiver: mpsc::Receiver<bool>,
    target: Target,
) -> Device<AsyncClient> {
    let mqtt = create_mqtt_connection(&thread_id, &target);
    Device::new(thread_id, receiver, target, mqtt)
}

impl<T: MqttClient> Device<T> {
    fn new(thread_id: u32, receiver: mpsc::Receiver<bool>, target: Target, mqtt: T) -> Self {
        let mut name = String::from("/device_");
        name.push_str(&thread_id.to_string());
        name.push_str("/");

        let seed = target.seed;
        let data_points = target.data_points;
        let generators = create_data_point_generators(seed, data_points);

        let wait_time_secs = target.wait_time_secs;
        Device {
            thread_id,
            name,
            generators,
            wait_time_secs,
            receiver,
            mqtt,
        }
    }

    pub fn run(&mut self) {
        loop {
            block_on(self.work());
            // I thought I could nap, but I probably shouldn't ... I seem to be blocking the MQTT client event loop despite being in another thread.
            if self.nap_or_stop() {
                break;
            }
        }
        debug!("Thread {} stopping", self.thread_id);
    }

    async fn work(&mut self) {
        debug!("Thread {} working", self.thread_id);
        let mut futures = Vec::new();
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        for generator in self.generators.iter_mut() {
            let (name, value) = generator.generate();

            let mut topic = self.name.clone();
            topic.push_str(name);

            let mut data = time.clone();
            data.push_str(",");
            data.push_str(&value.to_string());

            let f = self.mqtt.publish(topic, data);
            futures.push(f);
        }
        futures::future::join_all(futures).await;
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
        let generator = create_generator(GeneratorType::Status, rng.gen());
        generators.push(generator);
    }

    for _i in data_points / 3..2 * data_points / 3 {
        let generator = create_generator(GeneratorType::Noise, rng.gen());
        generators.push(generator);
    }

    for _i in 2 * data_points / 3..data_points {
        let generator = create_generator(GeneratorType::Sensor, rng.gen());
        generators.push(generator);
    }
    generators
}

fn create_mqtt_connection(thread_id: &u32, target: &Target) -> AsyncClient {
    let cap = target.data_points.into(); // TODO: Is this a good idea? We send all data points out for delivery at once.
    let thread_str = thread_id.to_string();
    let url = target.url.clone() + "?client_id=device_" + thread_str.as_str();
    let mut opts = MqttOptions::parse_url(url).unwrap();
    opts.set_credentials(&target.user, &target.pass);
    opts.set_keep_alive(Duration::from_secs(5));

    let (client, mut _eventloop) = AsyncClient::new(opts, cap);
    client
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::dummy_target;
    use mockall::predicate::*;
    use regex::Regex;

    // TODO Improve test coverage here
    #[test]
    fn test_publish() {
        let thread_id = 1;

        let (sender, receiver) = mpsc::channel();
        let _ = sender.send(true); // Make sure to send only one round of data.

        let target = dummy_target();

        let mut mock = MockMqttClient::new();

        let topic_check =
            function(|x: &String| Regex::new(r"^/device_[0-9]+/").unwrap().is_match(x));
        let data_check = function(|x: &String| Regex::new(r"^[0-9]+,[0-9]+").unwrap().is_match(x));

        mock.expect_publish()
            .with(topic_check, data_check)
            .times(usize::from(target.data_points))
            .returning(|_, _| ());

        let mut device = Device::new(thread_id, receiver, target, mock);
        device.run();
    }
}
