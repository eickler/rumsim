use std::time::Duration;

use crate::device::{DataPointIterator, Device};
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

    pub fn iter(&mut self) -> SimulationIterator {
        SimulationIterator {
            devices_iter: self.devices.iter_mut(),
            current_device_iter: None,
        }
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

pub struct SimulationIterator<'a> {
    devices_iter: std::slice::IterMut<'a, Device>,
    current_device_iter: Option<DataPointIterator<'a>>,
}

impl<'a> Iterator for SimulationIterator<'a> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(device_iter) = &mut self.current_device_iter {
                if let Some(data_point) = device_iter.next() {
                    return Some(data_point.clone());
                }
            }
            let next_device = self.devices_iter.next()?;
            self.current_device_iter = Some(next_device.iter());
        }
    }
}
