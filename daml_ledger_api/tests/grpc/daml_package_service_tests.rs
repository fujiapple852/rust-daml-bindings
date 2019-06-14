use crate::common::ping_pong::*;

use daml_ledger_api::data::package::{DamlHashFunction, DamlPackageStatus};

#[test]
fn test_list_packages() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock()?;
    let ledger_client = new_wallclock_sandbox()?;
    let is_found = ledger_client.package_service().list_packages_sync()?.iter().any(|p| p == PINGPONG_PACKAGE_ID);
    assert!(is_found);
    Ok(())
}

#[test]
fn test_get_package() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock()?;
    let ledger_client = new_wallclock_sandbox()?;
    let daml_package = ledger_client.package_service().get_package_sync(PINGPONG_PACKAGE_ID)?;
    assert!(!daml_package.payload().is_empty());
    assert_eq!(&DamlHashFunction::SHA256, daml_package.hash_function());
    assert_eq!(PINGPONG_PACKAGE_ID, daml_package.hash());
    Ok(())
}

#[test]
fn test_get_package_status() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock()?;
    let ledger_client = new_wallclock_sandbox()?;
    let daml_package_status = ledger_client.package_service().get_package_status_sync(PINGPONG_PACKAGE_ID)?;
    assert_eq!(DamlPackageStatus::Registered, daml_package_status);
    Ok(())
}
