use super::listener::ListenerOption;
use super::implant::ImplantOption;
use super::agent::AgentOption;

#[derive(Debug)]
pub struct Options {
    pub listener_opt: Option<ListenerOption>,
    pub agent_opt: Option<AgentOption>,
    pub implant_opt: Option<ImplantOption>,
}

impl Options {
    pub fn new() -> Self {
        Self {
            listener_opt: None,
            agent_opt: None,
            implant_opt: None,
        }
    }
}