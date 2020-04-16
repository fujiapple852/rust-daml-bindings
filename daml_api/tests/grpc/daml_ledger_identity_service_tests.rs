use crate::common::ping_pong::*;

#[tokio::test]
async fn test_get_ledger_identity() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock();
    let ledger_client = new_wallclock_sandbox().await?;
    let ledger_identity = ledger_client.ledger_identity_service().get_ledger_identity().await?;
    assert_eq!(ledger_identity, ledger_client.ledger_identity());
    Ok(())
}
