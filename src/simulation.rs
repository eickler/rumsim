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
    devices: Vec<Device>,
}

impl Simulation {
    pub fn new() -> Self {
        Simulation {
            devices: Vec::with_capacity(0),
        }
    }

    // TODO: Make this a real iterator!
    pub fn iter(&mut self) -> Vec<(String, String)> {
        self.devices.iter_mut().flat_map(|d| d.iter()).collect()
    }

    pub fn start(&mut self, param: SimulationParameters) {
        info!(
            "Starting simulation: {} devices, {} data points, {:?} wait time, {} seed",
            param.devices, param.data_points, param.wait_time, param.seed
        );

        self.devices.clear();
        self.devices = Vec::with_capacity(param.devices);

        let mut rng = StdRng::seed_from_u64(param.seed);
        for i in 0..param.devices {
            let device = Device::new(param.seed, i, param.data_points, rng.gen());
            self.devices.push(device);
        }
    }

    pub fn stop(&mut self) {
        info!("Stopping simulation.");
        self.devices.clear();
        self.devices = Vec::with_capacity(0);
    }
}
