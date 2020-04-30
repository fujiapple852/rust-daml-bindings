use crate::common::ping_pong::{new_static_sandbox, TestResult, ERR_STR, STATIC_SANDBOX_LOCK};
use daml_api::data::DamlLedgerConfiguration;
use futures::StreamExt;
use futures::TryStreamExt;
use std::time::Duration;

#[tokio::test]
async fn test_get_ledger_configuration() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let config_stream = ledger_client.ledger_configuration_service().get_ledger_configuration().await?;
    let all_config = config_stream.take(1).try_collect::<Vec<DamlLedgerConfiguration>>().await?;
    let config = all_config.first().ok_or(ERR_STR)?;
    assert_eq!(&Duration::new(86400, 0), config.max_deduplication_time());
    Ok(())
}
