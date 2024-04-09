use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Settings {
    // Simulation related settings
    pub sim_devices: usize,
    pub sim_data_points: usize,
    pub sim_frequency_secs: u64,
    pub sim_start_time: Option<DateTime<Utc>>,
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

fn get(env_variable: &str, default: &str) -> String {
    std::env::var(env_variable).unwrap_or(default.to_string())
}

fn get_num(env_variable: &str, default: usize) -> usize {
    std::env::var(env_variable)
        .unwrap_or(default.to_string())
        .parse()
        .unwrap() // It's OK to panic if someone sets a broken number in the environment.
}

fn get_time(env_variable: &str, default: Option<DateTime<Utc>>) -> Option<DateTime<Utc>> {
    std::env::var(env_variable)
        .ok()
        .map(|time| {
            DateTime::parse_from_rfc3339(&time)
                .unwrap()
                .with_timezone(&Utc)
        })
        .or(default)
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            // Simulation related settings
            sim_devices: get_num("SIM_DEVICES", 100),
            sim_data_points: get_num("SIM_DATA_POINTS", 100),
            sim_seed: get_num("SIM_SEED", 0) as u64,
            sim_frequency_secs: get_num("SIM_FREQUENCY_SECS", 1) as u64,
            sim_start_time: get_time("SIM_START_TIME", None),
            sim_runs: get_num("SIM_RUNS", usize::MAX),

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_variable() {
        std::env::set_var("TEST_VAR", "value");
        assert_eq!(get("TEST_VAR", "default"), "value");
        std::env::remove_var("TEST_VAR");
        assert_eq!(get("TEST_VAR", "default"), "default");
    }

    #[test]
    fn test_get_num_existing_variable() {
        std::env::set_var("TEST_NUM_VAR", "42");
        assert_eq!(get_num("TEST_NUM_VAR", 0), 42);
        std::env::remove_var("TEST_NUM_VAR");
        assert_eq!(get_num("TEST_NUM_VAR", 0), 0);
    }

    #[test]
    fn test_get_time_existing_variable() {
        std::env::set_var("TEST_TIME_VAR", "2022-01-01T00:00:00Z");
        let expected_time = DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(get_time("TEST_TIME_VAR", None), Some(expected_time));

        let result = std::panic::catch_unwind(|| {
            std::env::set_var("TEST_TIME_VAR", "Hans");
            get_time("TEST_TIME_VAR", None);
        });
        assert!(result.is_err());

        std::env::remove_var("TEST_TIME_VAR");
        assert_eq!(get_time("TEST_TIME_VAR", None), None);
    }
}
