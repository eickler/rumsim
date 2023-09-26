use async_trait::async_trait;
use rand::rngs::StdRng;
use rumqttc::QoS;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
use mockall::automock;

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

pub struct Device {
    name: String,
    generators: Vec<Box<dyn Generator>>,
}

impl Device {
    pub fn new(cluster_id: u16, device_id: u16, data_points: u16) -> Self {
        let name = format!("/device_{}_{}/", cluster_id, device_id);
        let generators = Self::create_data_point_generators(data_points);
        Device { name, generators }
    }

    pub async fn run<T: MqttClient>(&mut self, mqtt: &T, rng: &mut StdRng) {
        let mut futures = Vec::new();
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        for generator in self.generators.iter_mut() {
            let (name, value) = generator.generate(rng);

            let topic = format!("{}{}", self.name, name);
            let data = format!("{},{}", time, value);
            let f = mqtt.publish(topic, data);
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::{always, eq};
    use rand::SeedableRng;

    #[test]
    fn test_create_generators() {
        let mut rng = StdRng::seed_from_u64(1);

        let generators = Device::create_data_point_generators(0);
        assert_eq!(generators.len(), 0);

        let mut generators = Device::create_data_point_generators(1);
        assert_eq!(generators.len(), 1);
        let (name, _value) = generators[0].generate(&mut rng);
        assert!(name.contains("sensor"));

        let mut generators = Device::create_data_point_generators(2);
        assert_eq!(generators.len(), 2);
        let (name, _value) = generators[0].generate(&mut rng);
        assert!(name.contains("noise"));
        let (name, _value) = generators[1].generate(&mut rng);
        assert!(name.contains("sensor"));

        let mut generators = Device::create_data_point_generators(3);
        assert_eq!(generators.len(), 3);
        let (name, _value) = generators[0].generate(&mut rng);
        assert!(name.contains("status"));
        let (name, _value) = generators[1].generate(&mut rng);
        assert!(name.contains("noise"));
        let (name, _value) = generators[2].generate(&mut rng);
        assert!(name.contains("sensor"));

        let mut generators = Device::create_data_point_generators(4);
        assert_eq!(generators.len(), 4);
        let (name, _value) = generators[2].generate(&mut rng);
        assert!(name.contains("sensor"));
        let (name, _value) = generators[3].generate(&mut rng);
        assert!(name.contains("sensor"));
    }

    #[tokio::test]
    async fn test_run() {
        let data_points = 1;
        let mut device = Device::new(2, 3, data_points);

        let mut mock = MockMqttClient::new();
        mock.expect_publish()
            .with(eq(String::from("/device_2_3/sensor_0")), always())
            .times(usize::from(data_points))
            .returning(|_, _| ());
        let mut rng = StdRng::seed_from_u64(1);

        device.run(&mock, &mut rng).await;
    }
}
