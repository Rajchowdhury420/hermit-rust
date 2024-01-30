pub struct ListenerRoutes {
    pub home: String,
    pub register: String,
    pub task_ask: String,
    pub task_upload: String,
    pub task_result: String,
}

pub struct ListenerConfig {
    pub proto: String,
    pub host: String,
    pub port: u16,
    pub user_agent: String,
    pub routes: ListenerRoutes,
    pub https_root_cert: String,
    pub https_client_cert: String,
    pub https_client_key: String,
}

impl ListenerConfig {
    pub fn new(
        proto: String,
        host: String,
        port: u16,
        user_agent: String,
        https_root_cert: String,
        https_client_cert: String,
        https_client_key: String,
    ) -> Self {

        // TODO: Dynamic (randomly) routes
        let routes = ListenerRoutes {
            home: "/".to_string(),
            register: "/r".to_string(),
            task_ask: "/t/a".to_string(),
            task_upload: "/t/u".to_string(),
            task_result: "/t/r".to_string(),
        };
        
        Self {
            proto,
            host,
            port,
            user_agent,
            routes,
            https_root_cert,
            https_client_cert,
            https_client_key
        }
    }
}