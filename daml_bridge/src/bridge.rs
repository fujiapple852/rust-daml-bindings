use crate::aliases::{Archive, BridgeConfig, GrpcClient};
use crate::server::make_server;
use anyhow::Result;
use daml_grpc::DamlGrpcClientBuilder;
use daml_lf::element::DamlArchive;
use daml_util::package::{ArchiveAutoNamingStyle, DamlPackages};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{error, info};

/// Daml JSON<>GRPC API bridge.
pub struct Bridge {
    config: BridgeConfig,
}

impl Bridge {
    pub const fn new(config: BridgeConfig) -> Self {
        Self {
            config,
        }
    }

    /// Start the bridge.
    pub async fn run(&self) -> Result<()> {
        let grpc_client = Arc::new(
            DamlGrpcClientBuilder::uri(self.config.ledger_uri())
                .connect_timeout(Some(self.config.ledger_connect_timeout()))
                .timeout(self.config.ledger_timeout())
                .with_auth(self.config.ledger_token().to_owned())
                .connect()
                .await?,
        );
        let archive: Archive = Arc::new(RwLock::new(fetch_archive(&grpc_client).await?));
        let http_server = make_server(self.config.clone(), archive.clone(), grpc_client.clone())?;
        let package_refresher = refresh(archive.clone(), grpc_client.clone(), self.config.package_reload_interval());
        info!("Ready");
        let http_handle = tokio::spawn(http_server);
        let refresher_handle = tokio::spawn(package_refresher);
        let _result = tokio::join!(http_handle, refresher_handle);
        Ok(())
    }
}

/// Refresh the [`Archive`] from the ledger server.
async fn refresh(archive: Archive, grpc_client: GrpcClient, interval: Duration) {
    let mut timer = tokio::time::interval(interval);
    let _ = timer.tick().await;
    loop {
        let now = timer.tick().await;
        info!("refreshing Dar (Time now = {:?})", now);
        let new_archive = fetch_archive(&grpc_client).await;
        match new_archive {
            Ok(new_arch) => *archive.write().await = new_arch,
            Err(e) => {
                error!("error refreshing Dar from ledger: {}", e);
            },
        }
    }
}

async fn fetch_archive(grpc_client: &GrpcClient) -> Result<DamlArchive<'static>> {
    let all_packages = DamlPackages::from_ledger(grpc_client).await?;
    tokio::task::spawn_blocking(move || create_archive(all_packages)).await?
}

fn create_archive(packages: DamlPackages) -> Result<DamlArchive<'static>> {
    Ok(packages.into_dar(ArchiveAutoNamingStyle::Uuid)?.to_owned_archive()?)
}
