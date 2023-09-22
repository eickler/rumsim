use crate::commands::{Command, StartParam};
use crate::device::create_device;
use crate::settings::Target;
use log::{debug, warn};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct Control {
    target: Target,
    threads: Vec<thread::JoinHandle<()>>,
    channels: Vec<mpsc::Sender<bool>>,
}

impl Control {
    pub fn new(target: &Target) -> Self {
        Control {
            target: target.clone(),
            threads: Vec::new(),
            channels: Vec::new(),
        }
    }

    pub fn run(&mut self, cmd: &Command) {
        match cmd {
            Command::Start(start) => self.start(start),
            Command::Stop => self.stop(),
        }
    }

    // TODO: Could this be done just with async await instead of threads?
    // Since we anyway run the same wait time for all devices?
    fn start(&mut self, start_param: &StartParam) {
        self.stop();

        let num_threads = start_param.devices.into();
        let spawn_delay_millis =
            u64::from(start_param.wait_time_secs) * 1000 / u64::try_from(num_threads).unwrap();
        let spawn_delay = Duration::from_millis(spawn_delay_millis);

        for i in 0..num_threads {
            debug!("Starting thread {}", i);
            let (sender, receiver) = mpsc::channel();
            self.channels.push(sender);

            let target = self.target.clone();

            let handle = thread::spawn(move || {
                let mut device = create_device(i, receiver, target);
                device.run()
            });
            self.threads.push(handle);

            thread::sleep(spawn_delay);
        }
    }

    pub fn stop(&mut self) {
        for sender in self.channels.drain(..) {
            debug!("Sending stop signal");
            sender.send(true).unwrap();
        }

        for handle in self.threads.drain(..) {
            debug!("Waiting for exit");
            if let Err(err) = handle.join() {
                warn!("Failed to join thread: {:?}", err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::dummy_target;

    use super::*;

    #[test]
    fn test_start_stop_threads() {
        let mut control = Control::new(&dummy_target());
        let start_param = StartParam {
            devices: 10,
            data_points: 10,
            wait_time_secs: 5,
        };

        control.start(&start_param);
        assert_eq!(control.threads.len(), 10);

        control.stop();
        assert_eq!(control.threads.len(), 0);
    }

    #[test]
    fn test_restart_threads() {
        let mut control = Control::new(&dummy_target());
        let start_param = StartParam {
            devices: 2,
            data_points: 10,
            wait_time_secs: 2,
        };

        control.start(&start_param);
        assert_eq!(control.threads.len(), 2);

        let start_param_2 = StartParam {
            devices: 1,
            data_points: 10,
            wait_time_secs: 2,
        };
        control.start(&start_param_2);
        assert_eq!(control.threads.len(), 1);

        control.stop();
        assert_eq!(control.threads.len(), 0);
    }
}
