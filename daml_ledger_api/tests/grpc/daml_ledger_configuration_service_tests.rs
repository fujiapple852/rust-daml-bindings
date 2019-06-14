use crate::common::ping_pong::*;
use daml_ledger_api::data::DamlLedgerConfiguration;
use futures::future::Future;
use futures::stream::Stream;
use std::time::Duration;

#[test]
fn test_get_ledger_configuration() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;
    let config_future =
        ledger_client.ledger_configuration_service().get_ledger_configuration()?.take(1).collect().wait()?;
    let config: &DamlLedgerConfiguration = config_future.first().ok_or(ERR_STR)?;
    assert_eq!(&Duration::new(2, 0), config.min_ttl());
    assert_eq!(&Duration::new(30, 0), config.max_ttl());
    Ok(())
}
