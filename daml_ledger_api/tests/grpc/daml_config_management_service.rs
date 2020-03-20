use crate::common::ping_pong::*;
use chrono::{DateTime, Utc};
use daml_ledger_api::data::DamlTimeModel;
use std::time::Duration;

#[tokio::test]
async fn test_get_time_model() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock();
    let ledger_client = new_wallclock_sandbox().await?;
    let (configuration_generation, time_model) = ledger_client.config_management_service().get_time_model().await?;
    assert_eq!(0, configuration_generation);
    assert_eq!(Duration::from_secs(1), time_model.min_transaction_latency);
    assert_eq!(Duration::from_secs(1), time_model.max_clock_skew);
    assert_eq!(Duration::from_secs(30), time_model.max_ttl);
    Ok(())
}

#[tokio::test]
async fn test_set_time_model() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let maximum_record_time = "2019-01-02T03:45:56Z".parse::<DateTime<Utc>>()?;
    let new_time_model = DamlTimeModel::new(Duration::from_secs(2), Duration::from_secs(3), Duration::from_secs(25));
    let new_configuration_generation = ledger_client
        .config_management_service()
        .set_time_model(create_test_uuid(SUBMISSION_ID_PREFIX), maximum_record_time, 0, new_time_model)
        .await?;
    assert_eq!(1, new_configuration_generation);
    let (fetch_configuration_generation, fetch_time_model) =
        ledger_client.config_management_service().get_time_model().await?;
    assert_eq!(1, fetch_configuration_generation);
    assert_eq!(Duration::from_secs(2), fetch_time_model.min_transaction_latency);
    assert_eq!(Duration::from_secs(3), fetch_time_model.max_clock_skew);
    assert_eq!(Duration::from_secs(25), fetch_time_model.max_ttl);
    Ok(())
}
