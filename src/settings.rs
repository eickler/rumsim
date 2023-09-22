use config::{Config, ConfigError};
use serde::Deserialize;

const CONFIG_FILE: &str = "benchmark";

#[derive(Debug, Deserialize, Clone)]
pub struct Control {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub topic: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Target {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub devices: u32,
    pub data_points: u16,
    pub wait_time_secs: u16,
    pub seed: u64,
}

// Make this a method for generating?
pub fn dummy_target() -> Target {
    Target {
        url: String::from("mqtt://localhost:1883"),
        user: String::new(),
        pass: String::new(),
        devices: 1,
        data_points: 10,
        wait_time_secs: 1,
        seed: 1,
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub control: Control,
    pub target: Target,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let file = config::File::with_name(CONFIG_FILE);
        let settings = Config::builder().add_source(file).build()?;
        settings.try_deserialize()
    }
}
