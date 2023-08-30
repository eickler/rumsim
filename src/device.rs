use log::debug;
use std::sync::mpsc;
use std::time::Duration;

use crate::generator::Generator;

//   data_points_sensor, data_points_noise, data_points_status

pub struct Device {
    thread_id: i32,
    generators: Vec<&dyn Generator>,
    wait_time_secs: u16,
    receiver: mpsc::Receiver<bool>,
}

impl Device {
    pub fn new(
        thread_id: i32,
        data_points: u16,
        wait_time_secs: u16,
        receiver: mpsc::Receiver<bool>,
    ) -> Self {
        let mut generators = Vec::new();
        /*for i in 0..data_points {
            let mut generator = Generator::new();
            generators.push(generator);
        }*/

        Device {
            thread_id,
            generators,
            wait_time_secs,
            receiver,
        }
    }

    pub fn run(&mut self) {
        loop {
            /*
               Now I have to actually send in the loop via MQTT the device data points...
               does every device run an own client? Probably, for realistic conditions
               I should probably read the config somewhere beforehand...
            */
            debug!(
                "Thread {} working with parameter: {}",
                self.thread_id, self.data_points
            );

            let nap_time = Duration::from_secs(self.wait_time_secs.into());
            if let Ok(_msg) = self.receiver.recv_timeout(nap_time) {
                debug!("Thread {} stopping", self.thread_id);
                break;
            }
        }
    }
}
