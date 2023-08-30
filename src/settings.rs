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
