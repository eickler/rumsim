use async_trait::async_trait;
use rand::rngs::StdRng;
use rumqttc::QoS;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(not(test))]
use log::debug;
#[cfg(test)]
use mockall::automock;
#[cfg(test)]
use std::println as debug;

use crate::generator::{create_generator, Generator, GeneratorType};

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

// TODO Maybe rebuild this simply as wrapper? Somehow I cannot get this to work.
pub struct Device<T> {
    name: String,
    generators: Vec<Box<dyn Generator>>,
}

impl<T: MqttClient> Device<T> {
    pub fn new(cluster_id: u16, device_id: u16, data_points: u16) -> Self {
        let mut name = String::from("/device_");
        name.push_str(&device_id.to_string());
        name.push_str("/");

        let generators = Self::create_data_point_generators(data_points);

        Device { name, generators }
    }

    pub async fn run(&mut self, mqtt: T, rng: &StdRng) {
        let mut futures = Vec::new();
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        for generator in self.generators.iter_mut() {
            let (name, value) = generator.generate(rng);

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

    fn create_data_point_generators(data_points: u16) -> Vec<Box<dyn Generator>> {
        let mut generators = Vec::with_capacity(data_points.into());

        for i in 0..data_points / 3 {
            let generator = create_generator(GeneratorType::Status, i);
            generators.push(generator);
        }

        for i in data_points / 3..2 * data_points / 3 {
            let generator = create_generator(GeneratorType::Noise, i - data_points / 3);
            generators.push(generator);
        }

        for i in 2 * data_points / 3..data_points {
            let generator = create_generator(GeneratorType::Sensor, i - 2 * data_points / 3);
            generators.push(generator);
        }
        generators
    }
}
