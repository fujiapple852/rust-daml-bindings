use crate::config::BridgeConfigData;
use daml_grpc::DamlGrpcClient;
use daml_lf::element::DamlArchive;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A [`DamlArchive`] suitable for use by a multi-threaded async executor.
pub type Archive = Arc<RwLock<DamlArchive<'static>>>;

/// A [`DamlGrpcClient`] suitable for use by a multi-threaded async executor.
pub type GrpcClient = Arc<DamlGrpcClient>;

/// A [`BridgeConfigData`] suitable for use by a multi-threaded async executor.
pub type BridgeConfig = Arc<BridgeConfigData>;
