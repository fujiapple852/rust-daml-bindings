use crate::common::ping_pong::{new_wallclock_sandbox, TestResult, WALLCLOCK_SANDBOX_LOCK};

#[tokio::test]
async fn test_pruning() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock().await;
    let ledger_client = new_wallclock_sandbox().await?;
    ledger_client.participant_pruning_service().prune("00000000000000000000000000000000", None).await?;
    Ok(())
}
