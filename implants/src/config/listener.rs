pub struct ListenerConfig {
    pub proto: String,
    pub host: String,
    pub port: u16,
    pub user_agent: String,
}

impl ListenerConfig {
    pub fn new(proto: String, host: String, port: u16, user_agent: String) -> Self {
        Self { proto, host, port, user_agent }
    }
}