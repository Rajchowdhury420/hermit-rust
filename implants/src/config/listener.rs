pub struct ListenerConfig {
    pub proto: String,
    pub host: String,
    pub port: u16,
}

impl ListenerConfig {
    pub fn new(proto: String, host: String, port: u16) -> Self {
        Self { proto, host, port }
    }
}