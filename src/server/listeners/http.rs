pub struct HttpListener {
    pub id: u32,
}

impl HttpListener {
    pub fn new(id: u32) -> Self {
        Self {
            id
        }
    }
}