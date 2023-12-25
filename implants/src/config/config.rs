use super::listener::ListenerConfig;

pub struct Config {
    pub listener: ListenerConfig,
    pub sleep: u64,
}

impl Config {
    pub fn new(proto: String, host: String, port: u16, sleep: u64) -> Self {
        Self {
            listener: ListenerConfig::new(proto, host, port),
            sleep,
        }
    }
}