use std::time::Duration;

use crate::commands::{Command, SimulationParameters};
use crate::device::Device;
use log::info;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rumqttc::AsyncClient;

pub struct Simulation {
    param: SimulationParameters,
    devices: Vec<Device>,
    rng: StdRng,
}

impl Simulation {
    pub fn new() -> Self {
        Simulation {
            param: Self::init_params(),
            devices: Vec::with_capacity(0),
            rng: StdRng::seed_from_u64(1),
        }
    }

    fn init_params() -> SimulationParameters {
        SimulationParameters {
            devices: 0,
            data_points: 0,
            wait_time_secs: 1,
            seed: 1,
        }
    }

    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.param.wait_time_secs.into())
    }

    pub fn configure(&mut self, cmd: &Command) {
        match cmd {
            Command::Start(start) => self.start(start),
            Command::Stop => self.stop(),
        }
    }

    pub async fn run(&mut self, client: &AsyncClient) {
        for device in self.devices.iter_mut() {
            device.run(client, &mut self.rng).await;
        }
    }

    fn start(&mut self, param: &SimulationParameters) {
        info!(
            "Starting simulation: {} devices, {} data points, {} wait time, {} seed",
            param.devices, param.data_points, param.wait_time_secs, param.seed
        );
        self.param = param.clone();
        self.devices.clear();
        self.devices = Vec::with_capacity(self.param.devices.into());
        self.rng = StdRng::seed_from_u64(self.param.seed.into());

        for i in 0..self.param.devices {
            let device = Device::new(self.param.seed, i, self.param.data_points);
            self.devices.push(device);
        }
    }

    pub fn stop(&mut self) {
        info!("Stopping simulation.");
        self.param = Self::init_params();
        self.devices.clear();
        self.devices = Vec::with_capacity(0);
        self.rng = StdRng::seed_from_u64(1);
    }
}
