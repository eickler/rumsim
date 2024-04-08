use std::hash::{DefaultHasher, Hash, Hasher};

use crate::device::Device;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct Simulation {
    devices: Vec<Device>,
}

impl Simulation {
    pub fn new(client_id: &str, no_devices: usize, data_points: usize, seed: u64) -> Self {
        // Ensure that each instance of the simulator has a unique seed derived from the input seed and the instance ID.
        let mut hasher = DefaultHasher::new();
        client_id.hash(&mut hasher);
        seed.hash(&mut hasher);
        let mut rng = StdRng::seed_from_u64(hasher.finish());

        let mut devices = Vec::with_capacity(no_devices);
        for i in 0..no_devices {
            let device = Device::new(&client_id, i, data_points, rng.gen());
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
