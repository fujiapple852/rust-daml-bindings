use crate::common::ping_pong::{initialize_wallclock, new_wallclock_sandbox, TestResult};

#[tokio::test]
async fn test_pruning() -> TestResult {
    let _lock = initialize_wallclock().await;
    let ledger_client = new_wallclock_sandbox().await?;
    ledger_client.participant_pruning_service().prune("00000000000000000000000000000000", None).await?;
    Ok(())
}
