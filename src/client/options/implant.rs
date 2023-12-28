#[derive(Debug)]
pub struct ImplantOption {
    pub name: Option<String>,
    pub listener_url: Option<String>,
    pub os: Option<String>,
    pub arch: Option<String>,
    pub format: Option<String>,
    pub sleep: Option<u64>,
}