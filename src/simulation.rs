use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

use crate::device::Device;
use log::info;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SimulationParameters {
    pub devices: usize,
    pub data_points: usize,
    pub wait_time: Duration,
    pub seed: u64,
}

impl Default for SimulationParameters {
    fn default() -> Self {
        SimulationParameters {
            devices: 0,
            data_points: 0,
            wait_time: Duration::MAX,
            seed: 0,
        }
    }
}

pub struct Simulation {
    client_id: String,
    devices: Vec<Device>,
}

impl Simulation {
    pub fn new(client_id: &str) -> Self {
        Simulation {
            client_id: client_id.to_string(),
            devices: Vec::with_capacity(0),
        }
    }

    pub fn iter(&mut self) -> SimulationIterator {
        SimulationIterator {
            devices_iter: self.devices.iter_mut(),
        }
    }

    pub fn start(&mut self, param: SimulationParameters) {
        info!(
            "Starting simulation: {} devices, {} data points, {:?} wait time, {} seed",
            param.devices, param.data_points, param.wait_time, param.seed
        );

        self.devices.clear();
        self.devices = Vec::with_capacity(param.devices);

        // Ensure that each instance of the simulator has a unique seed derived from the input seed and the instance ID.
        let mut hasher = DefaultHasher::new();
        self.client_id.hash(&mut hasher);
        param.seed.hash(&mut hasher);
        let mut rng = StdRng::seed_from_u64(hasher.finish());

        for i in 0..param.devices {
            let device = Device::new(&self.client_id, i, param.data_points, rng.gen());
            self.devices.push(device);
        }
    }

    pub fn stop(&mut self) {
        info!("Stopping simulation.");
        self.devices.clear();
        self.devices = Vec::with_capacity(0);
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
