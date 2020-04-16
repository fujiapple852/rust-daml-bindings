use anyhow::Result;
use daml::api::DamlLedgerClientBuilder;
use std::io::Read;

pub(crate) async fn upload(dar_path: &str, uri: &str) -> Result<()> {
    let mut dar = std::fs::File::open(dar_path)?;
    let mut buffer = Vec::new();
    dar.read_to_end(&mut buffer)?;
    let ledger_client = DamlLedgerClientBuilder::uri(uri).connect().await?;
    ledger_client.package_management_service().upload_dar_file(buffer, None).await?;
    Ok(())
}
