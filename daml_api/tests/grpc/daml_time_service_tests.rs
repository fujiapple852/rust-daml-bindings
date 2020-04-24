use crate::common::ping_pong::{new_static_sandbox, TestResult, ERR_STR, STATIC_SANDBOX_LOCK};

use chrono::{DateTime, Utc};

use futures::StreamExt;
use futures::TryStreamExt;

#[tokio::test]
async fn test_get_time() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let ledger_times: Vec<DateTime<Utc>> = ledger_client.time_service().get_time().await?.take(1).try_collect().await?;
    let ledger_time = ledger_times.first().ok_or(ERR_STR)?;
    assert_eq!(&DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z")?, ledger_time);
    Ok(())
}

#[tokio::test]
async fn test_set_time() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let current_time = "1970-01-01T00:00:00Z".parse::<DateTime<Utc>>()?;
    let new_time = "2019-01-02T03:45:56Z".parse::<DateTime<Utc>>()?;
    ledger_client.time_service().set_time(current_time, new_time).await?;
    let ledger_times: Vec<DateTime<Utc>> = ledger_client.time_service().get_time().await?.take(1).try_collect().await?;
    let ledger_time = ledger_times.first().ok_or(ERR_STR)?;
    assert_eq!(&DateTime::parse_from_rfc3339("2019-01-02T03:45:56Z")?, ledger_time);
    Ok(())
}
