use super::listener::ListenerConfig;

pub struct Config {
    pub listener: ListenerConfig,
    pub sleep: u64,
    pub key: String,
    pub nonce: String,
}

impl Config {
    pub fn new(
        proto: String,
        host: String,
        port: u16,
        sleep: u64,
        key: String,
        nonce: String
    ) -> Self {
        Self {
            listener: ListenerConfig::new(proto, host, port),
            sleep,
            key,
            nonce,
        }
    }
}