use crate::common::ping_pong::{new_wallclock_sandbox, TestResult, PINGPONG_MODULE_NAME, WALLCLOCK_SANDBOX_LOCK};

use daml::util::package::find_module_package_id;
use daml_api::data::package::{DamlHashFunction, DamlPackageStatus};

#[tokio::test]
async fn test_list_packages() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock();
    let ledger_client = new_wallclock_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let is_found = ledger_client.package_service().list_packages().await?.iter().any(|p| p == &package_id);
    assert!(is_found);
    Ok(())
}

#[tokio::test]
async fn test_get_package() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock();
    let ledger_client = new_wallclock_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let daml_package = ledger_client.package_service().get_package(&package_id).await?;
    assert!(!daml_package.payload().is_empty());
    assert_eq!(&DamlHashFunction::SHA256, daml_package.hash_function());
    assert_eq!(&package_id, daml_package.hash());
    Ok(())
}

#[tokio::test]
async fn test_get_package_status() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock();
    let ledger_client = new_wallclock_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let daml_package_status = ledger_client.package_service().get_package_status(package_id).await?;
    assert_eq!(DamlPackageStatus::Registered, daml_package_status);
    Ok(())
}
