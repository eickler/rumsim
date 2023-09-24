use config::{Config, ConfigError};
use serde::Deserialize;

const CONFIG_FILE: &str = "benchmark";

#[derive(Debug, Deserialize, Clone)]
pub struct Control {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub control_topic: String,
    pub client_id: String,
    pub capacity: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub control: Control,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let file = config::File::with_name(CONFIG_FILE);
        let settings = Config::builder().add_source(file).build()?;
        settings.try_deserialize()

        // TODO: For Kubernetes, it makes much more sense to get the information from environment variables than from files..
    }
}
