use chrono::Utc;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::generator::{create_generator, Generator, GeneratorType};

pub struct Device {
    name: String,
    generators: Vec<Box<dyn Generator>>,
    rng: StdRng,
}

impl Device {
    /// Create a new device with the given cluster and device IDs and the number of data points.
    /// Cluster ID serves as a prefix for the device name to distinguish several simulators from each other.
    pub fn new(cluster_id: &str, device_id: usize, data_points: usize, seed: u64) -> Self {
        let name = format!("{}_{}", cluster_id, device_id);
        let generators = Self::create_data_point_generators(data_points);
        let rng = StdRng::seed_from_u64(seed);
        Device {
            name,
            generators,
            rng,
        }
    }

    /// Iterate over the data point generators and collect them into a string of the form
    /// 201,S,<time>,SF,<data point 1>,<value 1>,,SF,<data point 2>,<value 2>,,...
    /// What are the limitations here in terms of number of data points for C8Y?
    pub fn generate(&mut self) -> (String, String) {
        let topic = format!("s/us/{}", self.name);

        let current_time = Utc::now();
        let time_str = current_time.format("%+").to_string();

        let data = self
            .generators
            .iter_mut()
            .map(|generator| {
                let (datapoint, value) = generator.generate(&mut self.rng);
                format!("SF,{},{},", datapoint, value)
            })
            .collect::<Vec<String>>()
            .join(",");

        let message = format!("201,S,{},{}", time_str, data);
        (topic, message)
    }

    /// Each device produces roughly 1/3 of each type of data point, status, noise, and sensor data.
    fn create_data_point_generators(data_points: usize) -> Vec<Box<dyn Generator>> {
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
    async fn test_iter() {
        let data_points = 1;
        let mut device = Device::new("rumsim-2", 3, data_points, 1);
        let (topic, data) = device.generate();
        assert_eq!(topic, String::from("s/us/rumsim-2_3"));
        assert_eq!(data.split(',').count(), 7);
    }
}
