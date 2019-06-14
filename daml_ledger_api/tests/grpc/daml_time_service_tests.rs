use crate::common::ping_pong::*;

use chrono::{DateTime, Utc};
use futures::future::Future;
use futures::stream::Stream;

#[test]
fn test_get_time() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;
    let ledger_times: Vec<DateTime<Utc>> = ledger_client.time_service().get_time()?.take(1).collect().wait()?;
    let ledger_time = ledger_times.first().ok_or(ERR_STR)?;
    assert_eq!(&DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z")?, ledger_time);
    Ok(())
}

#[test]
fn test_set_time() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;
    let current_time = "1970-01-01T00:00:00Z".parse::<DateTime<Utc>>()?;
    let new_time = "2019-01-02T03:45:56Z".parse::<DateTime<Utc>>()?;
    ledger_client.time_service().set_time_sync(current_time, new_time)?;
    let ledger_times: Vec<DateTime<Utc>> = ledger_client.time_service().get_time()?.take(1).collect().wait()?;
    let ledger_time = ledger_times.first().ok_or(ERR_STR)?;
    assert_eq!(&DateTime::parse_from_rfc3339("2019-01-02T03:45:56Z")?, ledger_time);
    Ok(())
}
