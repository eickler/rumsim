use std::hash::{DefaultHasher, Hash, Hasher};

use crate::device::Device;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct SimulationParameters {
    pub client_id: String,
    pub devices: usize,
    pub data_points: usize,
    pub seed: u64,
    pub frequency_secs: u64,
    pub qos: u8,
}

pub struct Simulation {
    devices: Vec<Device>,
}

impl Simulation {
    pub fn new(parms: &SimulationParameters) -> Self {
        // Ensure that each instance of the simulator has a unique seed derived from the input seed and the instance ID.
        let mut hasher = DefaultHasher::new();
        parms.client_id.hash(&mut hasher);
        parms.seed.hash(&mut hasher);
        let mut rng = StdRng::seed_from_u64(hasher.finish());

        let mut devices = Vec::with_capacity(parms.devices);
        for i in 0..parms.devices {
            let device = Device::new(&parms.client_id, i, parms.data_points, rng.gen());
            devices.push(device);
        }

        Simulation { devices }
    }

    pub fn iter(&mut self) -> SimulationIterator {
        SimulationIterator {
            devices_iter: self.devices.iter_mut(),
        }
    }
}

pub struct SimulationIterator<'a> {
    devices_iter: std::slice::IterMut<'a, Device>,
}

impl<'a> Iterator for SimulationIterator<'a> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        self.devices_iter.next().map(|device| device.generate())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_new() {
        let devices = 1;
        let client_id = "test".to_string();
        let parms = SimulationParameters {
            client_id: client_id.clone(),
            devices,
            data_points: 1,
            seed: 12345,
            frequency_secs: 60,
            qos: 2,
        };

        let mut simulation = Simulation::new(&parms);
        assert_eq!(simulation.devices.len(), devices);

        let mut iter = simulation.iter();
        let (name, value) = iter.next().unwrap();
        assert!(name.contains(&client_id));
        assert!(name.contains("0")); // The device number of the first device.
        assert!(value.contains("sensor_0")); // The name of the first data point.
        assert!(value.contains("101.79")); // The value of the first data point with seed 12345.

        assert!(iter.next().is_none());
    }
}
