use log::info;

#[derive(Clone, Debug)]
pub struct Implant {
    pub id: u32,
    pub name: String,
    pub listener_url: String,
    pub os: String,
    pub arch: String,
    pub format: String,
    pub sleep: u16,
}

impl Implant {
    pub fn new(id: u32, name: String, listener_url: String, os: String, arch: String, format: String, sleep: u16) -> Self {
        Self {
            id,
            name,
            listener_url,
            os,
            arch,
            format,
            sleep,
        }
    }
}

pub fn format_implants(implants: &Vec<Implant>) -> String  {
    info!("Getting implants information...");
    if implants.len() == 0 {
        return String::from("No implants found.");
    }

    let mut output = format!(
        "{:>5} | {:<20} | {:<30} | {:<10} | {:<10} | {:<6} | {:>5}\n",
        "ID", "NAME", "LISTENER", "OS", "ARCH", "FORMAT", "SLEEP",
    );
    output = output + "-".repeat(108).as_str() + "\n";

    for implant in implants {
        output = output + format!(
            "{:>5} | {:<20} | {:<30} | {:<10} | {:<10} | {:<6} | {:>5}\n",
            implant.id.to_owned(),
            implant.name.to_owned(),
            implant.listener_url.to_owned(),
            implant.os.to_owned(),
            implant.arch.to_owned(),
            implant.format.to_owned(),
            implant.sleep.to_owned(),
        ).as_str();
    }

    return output;
}