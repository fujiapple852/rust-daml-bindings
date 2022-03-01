use crate::common::ping_pong::{
    create_test_uuid, initialize_wallclock, new_wallclock_sandbox, TestResult, SUBMISSION_ID_PREFIX,
};
use daml::lf::DarFile;
use daml_grpc::data::package::DamlPackageDetails;
use std::io::Read;

#[tokio::test]
async fn test_list_known_packages() -> TestResult {
    let _lock = initialize_wallclock().await;
    let ledger_client = new_wallclock_sandbox().await?;
    let known_packages = ledger_client.package_management_service().list_known_packages().await?;
    assert!(!known_packages.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_upload_dar_file() -> TestResult {
    let _lock = initialize_wallclock().await;
    let ledger_client = new_wallclock_sandbox().await?;
    let dar_file_path = "../resources/testing_types_sandbox/TestingTypes-latest.dar";
    let main_package_id = DarFile::from_file(dar_file_path)?.main.hash;
    let mut dar_file = std::fs::File::open(dar_file_path)?;
    let mut buffer = Vec::new();
    dar_file.read_to_end(&mut buffer)?;
    ledger_client
        .package_management_service()
        .upload_dar_file(buffer, Some(create_test_uuid(SUBMISSION_ID_PREFIX)))
        .await?;
    let known_packages = ledger_client.package_management_service().list_known_packages().await?;
    let found = known_packages.iter().map(DamlPackageDetails::package_id).find(|&id| id == main_package_id);
    assert!(found.is_some());
    Ok(())
}
