use crate::common::ping_pong::{initialize_wallclock, new_wallclock_sandbox, TestResult};
use daml_grpc::data::DamlFeaturesDescriptor;

#[tokio::test]
async fn test_get_version() -> TestResult {
    let _lock = initialize_wallclock().await;
    let ledger_client = new_wallclock_sandbox().await?;
    let (version, features) = ledger_client.version_service().get_ledger_api_version().await?;
    assert!(!version.is_empty());
    assert_eq!(features, Some(DamlFeaturesDescriptor::new(None)));
    Ok(())
}
