use daml_api::data::{DamlError, DamlResult};
use daml_api::DamlLedgerClient;
use daml_lf::DamlLfArchivePayload;

/// Return the id of a package which contains a given module name or en error if no such package exists.
///
/// The supplied `module_name` name is assumed to be in `DottedName` format, i.e. `TopModule.SubModule.Module`.
///
/// Implementation note: the packages are searched in the order returned by `DamlPackageService::list_packages()` but
/// this will change in future when package payload lookup is executed in parallel.
pub async fn find_module_package_id(ledger_client: &DamlLedgerClient, module_name: &str) -> DamlResult<String> {
    let all_packages = ledger_client.package_service().list_packages().await?;
    // TODO perform package payload lookup in parallel by spawning additional tasks and then await all (in any order).
    let mut all_archives = vec![];
    for package in &all_packages {
        all_archives.push(get_package_payload(ledger_client, package).await?);
    }
    all_archives
        .iter()
        .find(|(_, archive)| archive.contains_module(module_name))
        .map_or(Err("package could not be found".into()), |(package_id, _)| Ok((*package_id).to_string()))
}

#[allow(clippy::needless_lifetimes)]
async fn get_package_payload<'a>(
    ledger_client: &DamlLedgerClient,
    package_id: &'a str,
) -> DamlResult<(&'a str, DamlLfArchivePayload)> {
    let package = ledger_client.package_service().get_package(package_id).await?;
    let archive = DamlLfArchivePayload::from_bytes(package.payload).map_err(|e| DamlError::Other(e.to_string()))?;
    Ok((package_id, archive))
}
