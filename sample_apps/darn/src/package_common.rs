use anyhow::Result;
use daml::api::{DamlLedgerClientBuilder, DamlSandboxTokenBuilder};

use daml::lf::{DamlLfArchive, DamlLfArchivePayload, DamlLfHashFunction};
use futures::stream::FuturesUnordered;
use futures::TryStreamExt;
use std::sync::Arc;
use tokio::task::JoinHandle;
use uuid::Uuid;

const UNKNOWN_LF_ARCHIVE_PREFIX: &str = "__LF_ARCHIVE_NAME";

pub async fn get_all_packages(uri: &str, token_key_path: Option<&str>) -> Result<Vec<DamlLfArchive>> {
    let ledger_client = Arc::new(match token_key_path {
        Some(key) => DamlLedgerClientBuilder::uri(uri).with_auth(make_ec256_token(key)?).connect().await?,
        None => DamlLedgerClientBuilder::uri(uri).connect().await?,
    });
    let packages = ledger_client.package_management_service().list_known_packages().await?;
    let handles: FuturesUnordered<JoinHandle<Result<DamlLfArchive>>> = packages
        .iter()
        .map(|pd| {
            let ledger_client = ledger_client.clone();
            let pid = pd.package_id.clone();
            tokio::spawn(async move {
                let package = ledger_client.package_service().get_package(pid).await?;
                let archive = DamlLfArchivePayload::from_bytes(package.payload)?;
                let main = DamlLfArchive::new(
                    format!("{}-{}", UNKNOWN_LF_ARCHIVE_PREFIX, Uuid::new_v4()),
                    archive,
                    DamlLfHashFunction::SHA256,
                    package.hash,
                );
                Ok(main)
            })
        })
        .collect();
    handles.try_collect::<Vec<Result<DamlLfArchive>>>().await?.into_iter().collect::<Result<Vec<DamlLfArchive>>>()
}

fn make_ec256_token(token_key_path: &str) -> Result<String> {
    Ok(DamlSandboxTokenBuilder::new_with_duration_secs(30)
        .admin(true)
        .new_ec256_token(std::fs::read_to_string(token_key_path)?)?)
}
