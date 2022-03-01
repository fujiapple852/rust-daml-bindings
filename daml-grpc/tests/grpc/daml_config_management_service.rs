use crate::common::ping_pong::{
    create_test_uuid, initialize_static, initialize_wallclock, new_static_sandbox, new_wallclock_sandbox, TestResult,
    SUBMISSION_ID_PREFIX,
};
use chrono::{DateTime, Utc};
use daml_grpc::data::DamlTimeModel;
use futures::{StreamExt, TryStreamExt};
use std::ops::Add;
use std::time::Duration;

#[tokio::test]
async fn test_get_time_model() -> TestResult {
    let _lock = initialize_wallclock().await;
    let ledger_client = new_wallclock_sandbox().await?;
    let (configuration_generation, time_model) = ledger_client.config_management_service().get_time_model().await?;
    assert_eq!(1, configuration_generation);
    assert_eq!(Duration::from_secs(0), *time_model.avg_transaction_latency());
    assert_eq!(Duration::from_secs(120), *time_model.min_skew());
    assert_eq!(Duration::from_secs(120), *time_model.max_skew());
    Ok(())
}

#[tokio::test]
async fn test_set_time_model() -> TestResult {
    let _lock = initialize_static().await;
    let ledger_client = new_static_sandbox().await?;
    let ledger_times: Vec<DateTime<Utc>> = ledger_client.time_service().get_time().await?.take(1).try_collect().await?;
    let maximum_record_time = match ledger_times.as_slice() {
        [dt] => dt.add(chrono::Duration::seconds(5)),
        _ => panic!(),
    };
    let new_time_model = DamlTimeModel::new(Duration::from_secs(0), Duration::from_secs(30), Duration::from_secs(30));
    let new_configuration_generation = ledger_client
        .config_management_service()
        .set_time_model(create_test_uuid(SUBMISSION_ID_PREFIX), maximum_record_time, 1, new_time_model)
        .await?;
    assert_eq!(2, new_configuration_generation);
    let (fetch_configuration_generation, fetch_time_model) =
        ledger_client.config_management_service().get_time_model().await?;
    assert_eq!(2, fetch_configuration_generation);
    assert_eq!(Duration::from_secs(0), *fetch_time_model.avg_transaction_latency());
    assert_eq!(Duration::from_secs(30), *fetch_time_model.min_skew());
    assert_eq!(Duration::from_secs(30), *fetch_time_model.max_skew());
    Ok(())
}
