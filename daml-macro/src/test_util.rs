use chrono::{DateTime, NaiveDate, Utc};
use std::error::Error;

pub type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

pub fn make_date(date_str: &str) -> Result<NaiveDate, Box<dyn Error>> {
    Ok(NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?)
}

pub fn make_timestamp(timestamp_str: &str) -> Result<DateTime<Utc>, Box<dyn Error>> {
    Ok(timestamp_str.parse::<DateTime<Utc>>()?)
}
