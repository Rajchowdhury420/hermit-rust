use chrono::{Datelike, Timelike};

pub fn get_datetime(delimiter: &str) -> String {
    let now = chrono::offset::Utc::now();
    format!(
        "{}{}{}{}{}{}{}{}{}{}{}",
        now.year().to_string(),
        delimiter,
        now.month().to_string(),
        delimiter,
        now.day().to_string(),
        delimiter,
        now.hour().to_string(),
        delimiter,
        now.minute().to_string(),
        delimiter,
        now.second().to_string(),
    )
}