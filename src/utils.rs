// Utility functions, structs and constants
use edit_distance::edit_distance;
use std::time::SystemTime;
use chrono::{offset::TimeZone, Local, Date, NaiveDate};

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Event {
    pub movie_name: String,
    pub venue_code: String,
    pub date_string: String
}

// Add hash method for Event
impl Event {
    // New method
    pub fn new(movie_name: String, venue_code: String, date_string: String) -> Event {
        Event {
            movie_name,
            venue_code,
            date_string
        }
    }

    pub fn hash(&self) -> String {
        // TODO : Do sha256 hash of the event
        format!("{}:{}:{}", self.movie_name, self.venue_code, self.date_string)
    }
}



// Match percentage between strings using levenstein distance
pub fn match_percent(a: &str, b: &str) -> f32 {
    let a = a.to_lowercase();
    let b = b.to_lowercase();
    let dist = edit_distance(&a, &b);
    let max_len = std::cmp::max(a.len(), b.len());
    1.0 - (dist as f32) / (max_len as f32)
}


// General Utility functions
pub fn get_date_code(date: &Date<Local>) -> String{
    // Convert date of form dd-mm-yyyy to yyyymmdd
    let date_str = date.format("%Y%m%d").to_string();
    date_str
}

pub fn is_valid_date_string(date_str: &str) -> bool{
    let naive_date = NaiveDate::parse_from_str(date_str, "%d-%m-%Y");

    if naive_date.is_err(){
        return false;
    }
    // Successfully parsed, return true
    true
}

// Parse date string into date object
pub fn parse_date_string(date_str: &str) -> Date<Local>{
    let naive_date = NaiveDate::parse_from_str(date_str, "%d-%m-%Y").unwrap();
    Local.from_local_date(&naive_date).unwrap()
}

pub fn get_timestamp_as_millisecond() -> String{
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_micros()
        .to_string()
}

// Checks if a given date string is within the next 14 days
pub fn is_date_within_2_weeks(date_str: &str) -> bool{
    // Check if before today
    let date = parse_date_string(date_str);
    let now = Local::today();

    if date < now {
        return false;
    }

    let diff = date.signed_duration_since(now);
    let days = diff.num_days();
    if days > 14{
        return false;
    }

    true
}

// Check if a given date is in the past
pub fn is_date_in_past(date_str: &str) -> bool{
    let date = parse_date_string(date_str);
    let now = Local::today();

    if date < now {
        return true;
    }

    false
}