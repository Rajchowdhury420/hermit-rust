use log::info;

use crate::utils::str::truncated_format;

#[derive(Clone, Debug)]
pub struct Implant {
    pub id: u32,
    pub name: String,
    pub url: String,
    pub os: String,
    pub arch: String,
    pub format: String,
    pub sleep: u64,
    pub jitter: u64,
    pub user_agent: String,
}

impl Implant {
    pub fn new(
        id: u32,
        name: String,
        url: String,
        os: String,
        arch: String,
        format: String,
        sleep: u64,
        jitter: u64,
        user_agent: String,
    ) -> Self {
        Self {
            id,
            name,
            url,
            os,
            arch,
            format,
            sleep,
            jitter,
            user_agent,
        }
    }
}

pub fn format_implant_details(implant: Implant) -> String {
    info!("Getting the implant details...");

    let mut output = String::new();
    output = output + "\n";
    output = output + format!("{:<10} : {:<20}\n", "ID", implant.id).as_str();
    output = output + format!("{:<10} : {:<20}\n", "NAME", implant.name).as_str();
    output = output + format!("{:<10} : {:<20}\n", "LISTENER", implant.url).as_str();
    output = output + format!("{:<10} : {:<20}\n", "OS",
        format!("{}/{}", implant.os.to_owned(), implant.arch.to_owned())).as_str();
    output = output + format!("{:<10} : {:<20}\n", "FORMAT", implant.format).as_str();
    output = output + format!("{:<10} : {:<20}\n", "SLEEP", implant.sleep).as_str();
    output = output + format!("{:<10} : {:<20}\n", "JITTER", implant.jitter).as_str();
    output = output + format!("{:<10} : {:<20}\n", "USER AGENT", implant.user_agent).as_str();
    output
}

pub fn format_all_implants(implants: &Vec<Implant>) -> String  {
    info!("Getting implants information...");
    if implants.len() == 0 {
        return "Implants are empty".to_string();
    }

    let mut output = String::new();
    output = output + "\n";
    output = output + format!(
        "{:>5} | {:<20} | {:<26} | {:<18} | {:<6} | {:>5}\n",
        "ID", "NAME", "LISTENER", "OS", "FORMAT", "SLEEP",
    ).as_str();
    let output_len = output.len();
    output = output + "-".repeat(output_len).as_str() + "\n";

    for implant in implants {
        output = output + format!(
            "{:>5} | {:<20} | {:<26} | {:<18} | {:<6} | {:>5}\n",
            implant.id.to_owned(),
            truncated_format(implant.name.to_owned(), 17),
            truncated_format(implant.url.to_owned(), 23),
            truncated_format(format!("{}/{}", implant.os.to_owned(), implant.arch.to_owned()), 15),
            implant.format.to_owned(),
            implant.sleep.to_owned(),
        ).as_str();
    }

    return output;
}