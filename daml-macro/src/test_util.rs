use chrono::{Date, DateTime, NaiveDate, Utc};
use std::error::Error;

pub type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

pub fn make_date(date_str: &str) -> Result<Date<Utc>, Box<dyn Error>> {
    Ok(Date::<Utc>::from_utc(NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?, Utc))
}

pub fn make_timestamp(timestamp_str: &str) -> Result<DateTime<Utc>, Box<dyn Error>> {
    Ok(timestamp_str.parse::<DateTime<Utc>>()?)
}
