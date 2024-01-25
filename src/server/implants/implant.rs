use log::info;
use pad::Alignment;

use crate::utils::str::{
    table_format,
    TableItem,
};

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
    pub killdate: String,
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
        killdate: String,
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
            killdate,
        }
    }
}

pub fn format_implant_details(implant: Implant) -> String {
    info!("Getting the implant details...");

    let mut output = String::from("\n\n");
    output = output + format!("{:<10} : {}\n", "ID", implant.id).as_str();
    output = output + format!("{:<10} : {}\n", "NAME", implant.name).as_str();
    output = output + format!("{:<10} : {}\n", "LISTENER", implant.url).as_str();
    output = output + format!("{:<10} : {}\n", "OS",
        format!("{}/{}", implant.os.to_owned(), implant.arch.to_owned())).as_str();
    output = output + format!("{:<10} : {}\n", "FORMAT", implant.format).as_str();
    output = output + format!("{:<10} : {}\n", "SLEEP", implant.sleep).as_str();
    output = output + format!("{:<10} : {}\n", "JITTER", implant.jitter).as_str();
    output = output + format!("{:<10} : {}\n", "USER AGENT", implant.user_agent).as_str();
    output = output + format!("{:<10} : {}", "KILLDATE", implant.killdate).as_str();
    output
}

pub fn format_all_implants(implants: &Vec<Implant>) -> String  {
    info!("Getting implants information...");
    if implants.len() == 0 {
        return "Implants are empty".to_string();
    }

    let columns = vec![
        TableItem::new("ID".to_string(), 3, Alignment::Right, None),
        TableItem::new("NAME".to_string(), 14, Alignment::Left, None),
        TableItem::new("LISTENER".to_string(), 16, Alignment::Left, None),
        TableItem::new("OS".to_string(), 12, Alignment::Left, None),
        TableItem::new("FORMAT".to_string(), 6, Alignment::Left, None),
        TableItem::new("SLEEP".to_string(), 5, Alignment::Right, None),
    ];
    let mut rows: Vec<Vec<TableItem>> = Vec::new();
    for implant in implants {
        let row = vec![
            TableItem::new(implant.id.to_string(), 3, Alignment::Right, None),
            TableItem::new(implant.name.to_string(), 14, Alignment::Left, None),
            TableItem::new(implant.url.to_string(), 16, Alignment::Left, None),
            TableItem::new(
                format!("{}/{}", implant.os.to_string(), implant.arch.to_string()), 12, Alignment::Left, None),
            TableItem::new(implant.format.to_string(), 6, Alignment::Left, None),
            TableItem::new(implant.sleep.to_string(), 5, Alignment::Right, None),
        ];
        rows.push(row);
    }
    table_format(columns, rows)
}