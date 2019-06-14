use crate::data::{DamlError, DamlResult};
use chrono::Date;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use protobuf::well_known_types::Timestamp;
use std::time::UNIX_EPOCH;

#[allow(clippy::cast_possible_wrap)]
pub fn make_timestamp_secs(datetime: DateTime<Utc>) -> Timestamp {
    let mut timestamp = Timestamp::new();
    timestamp.set_seconds(datetime.timestamp());
    timestamp.set_nanos(datetime.timestamp_subsec_nanos() as i32);
    timestamp
}

#[allow(clippy::cast_sign_loss)]
pub fn make_datetime(timestamp: &Timestamp) -> DateTime<Utc> {
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp.get_seconds(), timestamp.get_nanos() as u32);
    DateTime::from_utc(naive_datetime, Utc)
}

pub fn date_from_days(days: i32) -> DamlResult<Date<Utc>> {
    Ok(DateTime::<Utc>::from(
        UNIX_EPOCH
            + time::Duration::days(i64::from(days)).to_std().map_err(|e| {
                DamlError::new_failed_conversion(format!("datetime from days {} out of range: {}", days, e.to_string()))
            })?,
    )
    .date())
}

pub fn datetime_from_micros(micros: i64) -> DamlResult<DateTime<Utc>> {
    Ok(DateTime::<Utc>::from(
        UNIX_EPOCH
            + time::Duration::microseconds(micros).to_std().map_err(|e| {
                DamlError::new_failed_conversion(format!(
                    "datetime from micros {} out of range: {}",
                    micros,
                    e.to_string()
                ))
            })?,
    ))
}

// TODO the lossy cast to i32 here...
#[allow(clippy::cast_possible_truncation)]
pub fn days_from_date(date: Date<Utc>) -> i32 {
    let duration: time::Duration = date.signed_duration_since(DateTime::<Utc>::from(UNIX_EPOCH).date());
    duration.num_days() as i32
}
