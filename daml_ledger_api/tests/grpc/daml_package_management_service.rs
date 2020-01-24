use crate::common::ping_pong::*;
use std::io::Read;

#[tokio::test]
async fn test_list_known_packages() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock()?;
    let ledger_client = new_wallclock_sandbox().await?;
    let known_packages = ledger_client.package_management_service().list_known_packages().await?;
    assert!(known_packages.len() > 0);
    Ok(())
}

#[tokio::test]
async fn test_upload_dar_file() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock()?;
    let ledger_client = new_wallclock_sandbox().await?;
    let dar_file_path = "../resources/testing_types_sandbox/archive/TestingTypes-1_0_0-sdk_0_13_36-lf_1_7.dar";
    let package_id = "ca1406c52d48b19fccbdb6996815cfbcad39d517a102f809db04f8257b4a6197";
    let mut dar_file = std::fs::File::open(dar_file_path)?;
    let mut buffer = Vec::new();
    dar_file.read_to_end(&mut buffer)?;
    ledger_client
        .package_management_service()
        .upload_dar_file(buffer, Some(create_test_uuid(SUBMISSION_ID_PREFIX)))
        .await?;
    let known_packages = ledger_client.package_management_service().list_known_packages().await?;
    let found = known_packages.into_iter().map(|p| p.package_id).find(|id| id == package_id);
    assert!(found.is_some());
    Ok(())
}
