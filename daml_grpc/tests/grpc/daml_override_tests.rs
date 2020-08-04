use crate::common::ping_pong::{make_ec256_token, new_wallclock_sandbox, TestResult, WALLCLOCK_SANDBOX_LOCK};

#[tokio::test]
async fn test_override_token() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock().await;
    let ledger_client = new_wallclock_sandbox().await?;
    let ledger_identity =
        ledger_client.ledger_identity_service().with_token(&make_ec256_token()?).get_ledger_identity().await?;
    assert_eq!(ledger_identity, ledger_client.ledger_identity());
    Ok(())
}

#[tokio::test]
async fn test_override_ledger_id() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock().await;
    let ledger_client = new_wallclock_sandbox().await?;
    let result = ledger_client.package_service().with_ledger_id("wallclock-sandbox").list_packages().await;
    assert!(result.is_ok());
    Ok(())
}
