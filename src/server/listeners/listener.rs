#[derive(Debug)]
pub enum ListenerMessage {
    Start,
    Stop,
}

#[derive(Clone, Debug)]
pub struct Listener {
    pub name: String,
    pub protocol: String,
    pub host: String,
    pub port: u16,
}

impl Listener {
    pub fn new(name: String, protocol: String, host: String, port: u16) -> Self {
        Self {
            name,
            protocol,
            host,
            port,
        }
    }
}