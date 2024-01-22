use chrono::{NaiveDateTime, Utc};

// Get the kill date as NaiveDateTime from String
pub fn get_killdate(killdate_str: &str) -> Option<NaiveDateTime> {
    match NaiveDateTime::parse_from_str(killdate_str, "%Y-%m-%d %H:%M:%S") {
        Ok(d) => Some(d),
        Err(_) => None,
    }
}

// Check whether the current datetime is past the Kill Date.
pub fn expires_killdate(killdate: Option<NaiveDateTime>, now: NaiveDateTime) -> bool {
    if let Some(kd) = killdate {
        (now - kd).num_seconds() > 0
    } else {
        // If the killdate is not set, return value is always false.
        false
    }
}