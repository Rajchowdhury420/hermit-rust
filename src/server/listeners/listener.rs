#[derive(Debug)]
pub struct Listener {
    pub id: u32,
    pub protocol: String,
    pub host: String,
    pub port: u16,
}

impl Listener {
    pub fn new(id: u32, protocol: String, host: String, port: u16) -> Self {
        Self {
            id,
            protocol,
            host,
            port,
        }
    }
}