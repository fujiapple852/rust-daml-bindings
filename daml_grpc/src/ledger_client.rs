use crate::data::{DamlError, DamlResult};
use crate::service::{
    DamlActiveContractsService, DamlCommandCompletionService, DamlCommandService, DamlCommandSubmissionService,
    DamlLedgerConfigurationService, DamlLedgerIdentityService, DamlPackageService, DamlParticipantPruningService,
    DamlTransactionService, DamlVersionService,
};
#[cfg(feature = "admin")]
use crate::service::{DamlConfigManagementService, DamlPackageManagementService, DamlPartyManagementService};
#[cfg(feature = "sandbox")]
use crate::service::{DamlResetService, DamlTimeService};
use std::time::Duration;
#[cfg(feature = "sandbox")]
use std::time::Instant;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tracing::{debug, instrument};

use hyper::client::HttpConnector;
#[cfg(test)]
use tonic::transport::Uri;

const DEFAULT_TIMEOUT_SECS: u64 = 5;
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 5;
#[cfg(feature = "sandbox")]
const DEFAULT_RESET_TIMEOUT_SECS: u64 = 5;

/// DOCME
#[derive(Debug, Default)]
pub struct DamlGrpcClientConfig {
    uri: String,
    timeout: Duration,
    connect_timeout: Option<Duration>,
    #[cfg(feature = "sandbox")]
    reset_timeout: Duration,
    concurrency_limit: Option<usize>,
    rate_limit: Option<(u64, Duration)>,
    initial_stream_window_size: Option<u32>,
    initial_connection_window_size: Option<u32>,
    tcp_keepalive: Option<Duration>,
    tcp_nodelay: bool,
    tls_config: Option<DamlGrpcTlsConfig>,
    auth_token: Option<String>,
}

/// DOCME
#[derive(Debug)]
pub struct DamlGrpcTlsConfig {
    ca_cert: Option<Vec<u8>>,
}

/// DOCME
pub struct DamlGrpcClientBuilder {
    config: DamlGrpcClientConfig,
}

impl DamlGrpcClientBuilder {
    /// DOCME
    pub fn uri(uri: impl Into<String>) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                uri: uri.into(),
                timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
                connect_timeout: Some(Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS)),
                #[cfg(feature = "sandbox")]
                reset_timeout: Duration::from_secs(DEFAULT_RESET_TIMEOUT_SECS),
                ..DamlGrpcClientConfig::default()
            },
        }
    }

    /// The network timeout.
    pub fn timeout(self, timeout: Duration) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                timeout,
                ..self.config
            },
        }
    }

    /// The connection timeout.
    pub fn connect_timeout(self, connect_timeout: Option<Duration>) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                connect_timeout,
                ..self.config
            },
        }
    }

    #[cfg(feature = "sandbox")]
    /// The sandbox reset timeout.
    pub fn reset_timeout(self, reset_timeout: Duration) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                reset_timeout,
                ..self.config
            },
        }
    }

    /// DOCME
    pub fn concurrency_limit(self, concurrency_limit: usize) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                concurrency_limit: Some(concurrency_limit),
                ..self.config
            },
        }
    }

    /// DOCME
    pub fn rate_limit(self, rate_limit: (u64, Duration)) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                rate_limit: Some(rate_limit),
                ..self.config
            },
        }
    }

    /// DOCME
    pub fn initial_stream_window_size(self, initial_stream_window_size: u32) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                initial_stream_window_size: Some(initial_stream_window_size),
                ..self.config
            },
        }
    }

    /// DOCME
    pub fn initial_connection_window_size(self, initial_connection_window_size: u32) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                initial_connection_window_size: Some(initial_connection_window_size),
                ..self.config
            },
        }
    }

    /// DOCME
    pub fn tcp_keepalive(self, tcp_keepalive: Duration) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                tcp_keepalive: Some(tcp_keepalive),
                ..self.config
            },
        }
    }

    /// DOCME
    pub fn tcp_nodelay(self, tcp_nodelay: bool) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                tcp_nodelay,
                ..self.config
            },
        }
    }

    /// DOCME
    pub fn with_tls(self, ca_cert: impl Into<Vec<u8>>) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                tls_config: Some(DamlGrpcTlsConfig {
                    ca_cert: Some(ca_cert.into()),
                }),
                ..self.config
            },
        }
    }

    /// DOCME
    pub fn with_auth(self, auth_token: String) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                auth_token: Some(auth_token),
                ..self.config
            },
        }
    }

    /// DOCME
    pub async fn connect(self) -> DamlResult<DamlGrpcClient> {
        DamlGrpcClient::connect(self.config).await
    }
}

/// DAML ledger client connection.
#[derive(Debug)]
pub struct DamlGrpcClient {
    config: DamlGrpcClientConfig,
    channel: Channel,
    ledger_identity: String,
}

impl DamlGrpcClient {
    /// Create a channel and connect.
    #[instrument]
    pub async fn connect(config: DamlGrpcClientConfig) -> DamlResult<Self> {
        debug!("connecting to {}", config.uri);
        let channel = Self::open_channel(&config).await?;
        Self::make_client_from_channel(channel, config).await
    }

    /// Reset the ledger and reconnect.
    #[cfg(feature = "sandbox")]
    #[instrument(skip(self))]
    pub async fn reset_and_wait(self) -> DamlResult<Self> {
        debug!("resetting Sandbox");
        self.reset_service().reset().await?;
        let channel = Self::open_channel_and_wait(&self.config).await?;
        Self::make_client_from_channel_and_wait(channel, self.config).await
    }

    /// Return the current configuration.
    pub const fn config(&self) -> &DamlGrpcClientConfig {
        &self.config
    }

    /// DOCME
    pub fn ledger_identity(&self) -> &str {
        &self.ledger_identity
    }

    /// DOCME
    pub fn ledger_identity_service(&self) -> DamlLedgerIdentityService<'_> {
        DamlLedgerIdentityService::new(self.channel.clone(), self.config.auth_token.as_deref())
    }

    /// DOCME
    pub fn ledger_configuration_service(&self) -> DamlLedgerConfigurationService<'_> {
        DamlLedgerConfigurationService::new(
            self.channel.clone(),
            &self.ledger_identity,
            self.config.auth_token.as_deref(),
        )
    }

    /// DOCME
    pub fn package_service(&self) -> DamlPackageService<'_> {
        DamlPackageService::new(self.channel.clone(), &self.ledger_identity, self.config.auth_token.as_deref())
    }

    /// DOCME
    pub fn command_submission_service(&self) -> DamlCommandSubmissionService<'_> {
        DamlCommandSubmissionService::new(
            self.channel.clone(),
            &self.ledger_identity,
            self.config.auth_token.as_deref(),
        )
    }

    /// DOCME
    pub fn command_completion_service(&self) -> DamlCommandCompletionService<'_> {
        DamlCommandCompletionService::new(
            self.channel.clone(),
            &self.ledger_identity,
            self.config.auth_token.as_deref(),
        )
    }

    /// DOCME
    pub fn command_service(&self) -> DamlCommandService<'_> {
        DamlCommandService::new(self.channel.clone(), &self.ledger_identity, self.config.auth_token.as_deref())
    }

    /// DOCME
    pub fn transaction_service(&self) -> DamlTransactionService<'_> {
        DamlTransactionService::new(self.channel.clone(), &self.ledger_identity, self.config.auth_token.as_deref())
    }

    /// DOCME
    pub fn active_contract_service(&self) -> DamlActiveContractsService<'_> {
        DamlActiveContractsService::new(self.channel.clone(), &self.ledger_identity, self.config.auth_token.as_deref())
    }

    /// DOCME
    pub fn version_service(&self) -> DamlVersionService<'_> {
        DamlVersionService::new(self.channel.clone(), &self.ledger_identity, self.config.auth_token.as_deref())
    }

    /// DOCME
    #[cfg(feature = "admin")]
    pub fn package_management_service(&self) -> DamlPackageManagementService<'_> {
        DamlPackageManagementService::new(self.channel.clone(), self.config.auth_token.as_deref())
    }

    /// DOCME
    #[cfg(feature = "admin")]
    pub fn party_management_service(&self) -> DamlPartyManagementService<'_> {
        DamlPartyManagementService::new(self.channel.clone(), self.config.auth_token.as_deref())
    }

    /// DOCME
    #[cfg(feature = "admin")]
    pub fn config_management_service(&self) -> DamlConfigManagementService<'_> {
        DamlConfigManagementService::new(self.channel.clone(), self.config.auth_token.as_deref())
    }

    /// DOCME
    #[cfg(feature = "admin")]
    pub fn participant_pruning_service(&self) -> DamlParticipantPruningService<'_> {
        DamlParticipantPruningService::new(self.channel.clone(), self.config.auth_token.as_deref())
    }

    /// DOCME
    #[cfg(feature = "sandbox")]
    pub fn reset_service(&self) -> DamlResetService<'_> {
        DamlResetService::new(self.channel.clone(), &self.ledger_identity, self.config.auth_token.as_deref())
    }

    /// DOCME
    #[cfg(feature = "sandbox")]
    pub fn time_service(&self) -> DamlTimeService<'_> {
        DamlTimeService::new(self.channel.clone(), &self.ledger_identity, self.config.auth_token.as_deref())
    }

    async fn open_channel(config: &DamlGrpcClientConfig) -> DamlResult<Channel> {
        Self::make_channel(config).await
    }

    async fn make_channel(config: &DamlGrpcClientConfig) -> DamlResult<Channel> {
        let mut channel = Channel::from_shared(config.uri.clone())?;
        if let Some(limit) = config.concurrency_limit {
            channel = channel.concurrency_limit(limit);
        }
        if let Some((limit, duration)) = config.rate_limit {
            channel = channel.rate_limit(limit, duration);
        }
        if let Some(size) = config.initial_stream_window_size {
            channel = channel.initial_connection_window_size(size);
        }
        if let Some(size) = config.initial_connection_window_size {
            channel = channel.initial_connection_window_size(size);
        }
        if let Some(duration) = config.tcp_keepalive {
            channel = channel.tcp_keepalive(Some(duration));
        }
        channel = channel.tcp_nodelay(config.tcp_nodelay);
        match &config.tls_config {
            Some(DamlGrpcTlsConfig {
                ca_cert: Some(cert),
            }) => {
                channel = channel.tls_config(ClientTlsConfig::new().ca_certificate(Certificate::from_pem(cert)))?;
            },
            Some(DamlGrpcTlsConfig {
                ca_cert: None,
            }) => {
                channel = channel.tls_config(ClientTlsConfig::new())?;
            },
            _ => {},
        }

        // Tonic does not current allow us to set a connect timeout (see https://github.com/hyperium/tonic/issues/498)
        // directly and so we workaround this by creating the Hyper HttpConnector directly.
        let mut http = HttpConnector::new();
        http.enforce_http(false);
        http.set_nodelay(config.tcp_nodelay);
        http.set_keepalive(config.tcp_keepalive);
        http.set_connect_timeout(config.connect_timeout);
        channel.connect_with_connector(http).await.map_err(DamlError::from)
    }

    async fn make_client_from_channel(channel: Channel, config: DamlGrpcClientConfig) -> DamlResult<Self> {
        let ledger_identity_service = DamlLedgerIdentityService::new(channel.clone(), config.auth_token.as_deref());
        let ledger_identity = ledger_identity_service.get_ledger_identity().await?;
        Ok(Self {
            config,
            channel: channel.clone(),
            ledger_identity,
        })
    }

    #[cfg(feature = "sandbox")]
    async fn open_channel_and_wait(config: &DamlGrpcClientConfig) -> DamlResult<Channel> {
        let mut channel = Self::make_channel(config).await;
        let start = Instant::now();
        while let Err(e) = channel {
            if start.elapsed() > config.reset_timeout {
                return Err(DamlError::new_timeout_error(e));
            }
            channel = Self::make_channel(config).await;
        }
        channel
    }

    #[cfg(feature = "sandbox")]
    async fn make_client_from_channel_and_wait(channel: Channel, config: DamlGrpcClientConfig) -> DamlResult<Self> {
        let ledger_identity_service = DamlLedgerIdentityService::new(channel.clone(), config.auth_token.as_deref());
        let ledger_identity =
            Self::query_ledger_identity_and_wait(&config.reset_timeout, &ledger_identity_service).await?;
        Ok(Self {
            config,
            channel: channel.clone(),
            ledger_identity,
        })
    }

    #[cfg(feature = "sandbox")]
    async fn query_ledger_identity_and_wait(
        reset_timeout: &Duration,
        ledger_identity_service: &DamlLedgerIdentityService<'_>,
    ) -> DamlResult<String> {
        let start = Instant::now();
        let mut ledger_identity: DamlResult<String> = ledger_identity_service.get_ledger_identity().await;
        while let Err(e) = ledger_identity {
            if let DamlError::GprcPermissionError(_) = e {
                return Err(e);
            }
            if start.elapsed() > *reset_timeout {
                return Err(DamlError::new_timeout_error(e));
            }
            ledger_identity = ledger_identity_service.get_ledger_identity().await;
        }
        ledger_identity
    }

    #[cfg(test)]
    pub(crate) async fn dummy_for_testing() -> Self {
        DamlGrpcClient {
            config: DamlGrpcClientConfig::default(),
            channel: Channel::builder(Uri::from_static("http://dummy.for.testing")).connect_lazy(),
            ledger_identity: String::default(),
        }
    }
}
