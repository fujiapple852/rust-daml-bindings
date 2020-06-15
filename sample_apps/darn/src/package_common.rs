use anyhow::Result;
use daml::grpc_api::data::package::DamlPackage;
use daml::grpc_api::{DamlGrpcClientBuilder};
use daml::util::DamlSandboxTokenBuilder;
use futures::stream::FuturesUnordered;
use futures::TryStreamExt;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub async fn get_all_packages(uri: &str, token_key_path: Option<&str>) -> Result<Vec<DamlPackage>> {
    let ledger_client = Arc::new(match token_key_path {
        Some(key) => DamlGrpcClientBuilder::uri(uri).with_auth(make_ec256_token(key)?).connect().await?,
        None => DamlGrpcClientBuilder::uri(uri).connect().await?,
    });
    let packages = ledger_client.package_management_service().list_known_packages().await?;
    let handles: FuturesUnordered<JoinHandle<Result<DamlPackage>>> = packages
        .iter()
        .map(|pd| {
            let ledger_client = ledger_client.clone();
            let pid = pd.package_id().to_owned();
            tokio::spawn(async move {
                let package = ledger_client.package_service().get_package(pid).await?;

                Ok(package)
            })
        })
        .collect();
    handles.try_collect::<Vec<Result<DamlPackage>>>().await?.into_iter().collect::<Result<Vec<DamlPackage>>>()
}

pub fn make_ec256_token(token_key_path: &str) -> Result<String> {
    Ok(DamlSandboxTokenBuilder::new_with_duration_secs(30)
        .admin(true)
        .new_ec256_token(std::fs::read_to_string(token_key_path)?)?)
}
