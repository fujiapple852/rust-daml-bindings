use crate::common::ping_pong::*;

#[test]
fn test_get_ledger_identity() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock()?;
    let ledger_client = new_wallclock_sandbox()?;
    let ledger_identity = ledger_client.ledger_identity_service().get_ledger_identity_sync()?;
    assert_eq!(ledger_identity, ledger_client.ledger_identity());
    Ok(())
}
