#[derive(Debug, Clone)]
pub struct Settings {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub control_topic: String,
    pub client_id: String,
    pub capacity: usize,
    pub qos: u8,
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
            url: get("URL", "mqtt://localhost:1883"),
            user: get("USER", "mqtt"),
            pass: get("PASS", "pass"),
            client_id: get("CLIENT_ID", "rumsim-0"),
            control_topic: get("CONTROL_TOPIC", "control"),
            capacity: get_num("CAPACITY", 1000),
            qos: get_num("QOS", 1) as u8,
        }
    }
}
