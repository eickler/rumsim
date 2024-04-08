#[derive(Debug, Clone)]
pub struct Settings {
    // Simulation related settings
    pub sim_devices: usize,
    pub sim_data_points: usize,
    pub sim_frequency_secs: u64,
    pub sim_wait_time_secs: u64,
    pub sim_runs: usize,
    pub sim_seed: u64,

    // MQTT related settings
    pub broker_url: String,
    pub broker_user: String,
    pub broker_pass: String,
    pub broker_client_id: String,
    pub broker_qos: u8,

    // Observability related settings
    pub otlp_collector: Option<String>,
    pub otlp_auth: Option<String>,

    // Other parameters
    pub capacity: usize,
}

pub fn get(env_variable: &str, default: &str) -> String {
    std::env::var(env_variable).unwrap_or(default.to_string())
}

pub fn get_num(env_variable: &str, default: usize) -> usize {
    std::env::var(env_variable)
        .unwrap_or(default.to_string())
        .parse()
        .unwrap() // It's OK to panic if someone sets a broken number in the environment.
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            // Simulation related settings
            sim_devices: get_num("SIM_DEVICES", 100),
            sim_data_points: get_num("SIM_DATA_POINTS", 100),
            sim_seed: get_num("SIM_SEED", 0) as u64,
            sim_frequency_secs: get_num("SIM_FREQUENCY_SECS", 1) as u64,
            sim_wait_time_secs: get_num("SIM_WAIT_TIME_SECS", 0) as u64,
            sim_runs: get_num("SIM_RUNS", 0),

            // MQTT related settings
            broker_url: get("BROKER_URL", "mqtt://localhost:1883"),
            broker_user: get("BROKER_USER", "mqtt"),
            broker_pass: get("BROKER_PASS", "pass"),
            broker_client_id: get("BROKER_CLIENT_ID", "rumsim-0"),
            broker_qos: get_num("BROKER_QOS", 1) as u8,

            // Observability related settings
            otlp_collector: std::env::var("OTLP_ENDPOINT").ok(),
            otlp_auth: std::env::var("OLTP_AUTH").ok(),

            // Other parameters
            capacity: get_num("CAPACITY", 1000),
        }
    }
}
