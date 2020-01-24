use crate::data::{DamlError, DamlResult};
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use chrono::{Date, Timelike};
use std::convert::TryFrom;
use std::error::Error;
use std::time::{Duration, UNIX_EPOCH};

#[allow(clippy::cast_sign_loss)]
pub fn from_grpc_timestamp(timestamp: &prost_types::Timestamp) -> DateTime<Utc> {
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32);
    DateTime::from_utc(naive_datetime, Utc)
}

pub fn to_grpc_timestamp(datetime: DateTime<Utc>) -> DamlResult<prost_types::Timestamp> {
    Ok(prost_types::Timestamp {
        seconds: datetime.timestamp(),
        nanos: i32::try_from(datetime.nanosecond()).map_err(|e| DamlError::new_failed_conversion(e.description()))?,
    })
}

#[allow(clippy::cast_sign_loss)]
pub fn from_grpc_duration(duration: &prost_types::Duration) -> Duration {
    Duration::new(duration.seconds as u64, duration.nanos as u32)
}

pub fn to_grpc_duration(duration: &Duration) -> DamlResult<prost_types::Duration> {
    Ok(prost_types::Duration {
        seconds: i64::try_from(duration.as_secs()).map_err(|e| DamlError::new_failed_conversion(e.description()))?,
        nanos: i32::try_from(duration.subsec_nanos()).map_err(|e| DamlError::new_failed_conversion(e.description()))?,
    })
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

/// Required value.
pub trait Required<T> {
    fn req(self) -> DamlResult<T>;
}

impl<T> Required<T> for Option<T> {
    fn req(self) -> DamlResult<T> {
        self.ok_or_else(|| DamlError::MissingRequiredField)
    }
}
