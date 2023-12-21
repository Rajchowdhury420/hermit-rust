use super::listener::ListenerOption;
use super::implant::ImplantOption;

#[derive(Debug)]
pub struct Options {
    pub listener_opt: Option<ListenerOption>,
    pub implant_opt: Option<ImplantOption>,
}

impl Options {
    pub fn new() -> Self {
        Self {
            listener_opt: None,
            implant_opt: None,
        }
    }
}