use config::{Config, ConfigError};
use serde::Deserialize;

const CONFIG_FILE: &str = "defaults";

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub control_topic: String,
    pub client_id: String,
    pub capacity: usize,
    pub qos: u8,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let file = config::File::with_name(CONFIG_FILE);
        let environment = config::Environment::default();
        let settings = Config::builder()
            .add_source(file)
            .add_source(environment)
            .build()?;
        settings.try_deserialize()
    }
}
