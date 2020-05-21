use crate::package_common::make_ec256_token;
use anyhow::Result;
use daml::api::DamlLedgerClientBuilder;
use std::io::Read;

pub async fn upload(dar_path: &str, uri: &str, token_key_path: Option<&str>) -> Result<()> {
    let mut dar = std::fs::File::open(dar_path)?;
    let mut buffer = Vec::new();
    dar.read_to_end(&mut buffer)?;

    let ledger_client = match token_key_path {
        Some(key) => DamlLedgerClientBuilder::uri(uri).with_auth(make_ec256_token(key)?).connect().await?,
        None => DamlLedgerClientBuilder::uri(uri).connect().await?,
    };

    // let ledger_client = DamlLedgerClientBuilder::uri(uri).connect().await?;
    ledger_client.package_management_service().upload_dar_file(buffer, None).await?;

    // TODO verify upload worked as server doesn't give us a sync error!

    Ok(())
}
