use rand::rngs::StdRng;
use rand::SeedableRng;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::generator::{create_generator, Generator, GeneratorType};

pub struct Device {
    name: String,
    generators: Vec<Box<dyn Generator>>,
    rng: StdRng,
}

impl Device {
    /// Create a new device with the given cluster and device IDs and the number of data points.
    /// Cluster ID serves as a prefix for the device name to distinguish several simulators from each other.
    pub fn new(cluster_id: u64, device_id: usize, data_points: usize, seed: u64) -> Self {
        let name = format!("/device_{}_{}/", cluster_id, device_id);
        let generators = Self::create_data_point_generators(data_points);
        let rng = StdRng::seed_from_u64(seed);
        Device {
            name,
            generators,
            rng,
        }
    }

    pub fn iter(&mut self) -> DataPointIterator {
        DataPointIterator {
            device: self,
            index: 0,
        }
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

pub struct DataPointIterator<'a> {
    device: &'a mut Device,
    index: usize,
}

impl<'a> Iterator for DataPointIterator<'a> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.device.generators.len() {
            let generator = &mut self.device.generators[self.index];
            let (name, value) = generator.generate(&mut self.device.rng);

            let topic = format!("{}{}", self.device.name, name);
            let data = format!("{},{}", get_time(), value);

            self.index += 1;

            Some((topic, data))
        } else {
            None
        }
    }
}

pub fn get_time() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
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
        let mut device = Device::new(2, 3, data_points, 1);
        let mut iter = device.iter();
        let (topic, data) = iter.next().unwrap();
        assert_eq!(topic, String::from("/device_2_3/sensor_0"));
        assert_eq!(data.split(',').count(), 2);
        assert!(iter.next().is_none());
    }
}
