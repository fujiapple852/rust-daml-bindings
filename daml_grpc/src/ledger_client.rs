use crate::data::{DamlError, DamlResult};
use crate::service::{
    DamlActiveContractsService, DamlCommandCompletionService, DamlCommandService, DamlCommandSubmissionService,
    DamlLedgerConfigurationService, DamlLedgerIdentityService, DamlPackageService, DamlTransactionService,
};
#[cfg(feature = "admin")]
use crate::service::{DamlConfigManagementService, DamlPackageManagementService, DamlPartyManagementService};
#[cfg(feature = "sandbox")]
use crate::service::{DamlResetService, DamlTimeService};
use log::debug;
use std::time::{Duration, Instant};
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

const DEFAULT_TIMEOUT_SECS: u64 = 5;

/// DOCME
#[derive(Debug, Default)]
pub struct DamlGrpcClientConfig {
    uri: String,
    timeout: Duration,
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
                ..DamlGrpcClientConfig::default()
            },
        }
    }

    /// DOCME
    pub fn timeout(self, timeout: Duration) -> Self {
        Self {
            config: DamlGrpcClientConfig {
                timeout,
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
pub struct DamlGrpcClient {
    config: DamlGrpcClientConfig,
    ledger_identity: String,
    ledger_identity_service: DamlLedgerIdentityService,
    ledger_configuration_service: DamlLedgerConfigurationService,
    package_service: DamlPackageService,
    command_submission_service: DamlCommandSubmissionService,
    command_completion_service: DamlCommandCompletionService,
    command_service: DamlCommandService,
    transaction_service: DamlTransactionService,
    active_contract_service: DamlActiveContractsService,
    #[cfg(feature = "admin")]
    package_management_service: DamlPackageManagementService,
    #[cfg(feature = "admin")]
    party_management_service: DamlPartyManagementService,
    #[cfg(feature = "admin")]
    config_management_service: DamlConfigManagementService,
    #[cfg(feature = "sandbox")]
    reset_service: DamlResetService,
    #[cfg(feature = "sandbox")]
    time_service: DamlTimeService,
}

impl DamlGrpcClient {
    /// Create a channel and connect.
    pub async fn connect(config: DamlGrpcClientConfig) -> DamlResult<Self> {
        debug!("connecting to {}", config.uri);
        let channel = Self::open_channel(&config).await?;
        Self::make_client_from_channel(&channel, config).await
    }

    /// Reset the ledger and reconnect.
    #[cfg(feature = "sandbox")]
    pub async fn reset_and_wait(self) -> DamlResult<Self> {
        debug!("resetting Sandbox");
        self.reset_service.reset().await?;
        let channel = Self::open_channel(&self.config).await?;
        Self::make_client_from_channel(&channel, self.config).await
    }

    /// Refresh the authentication token used by this client.
    pub fn refresh_token(self, new_token: impl Into<String>) -> Self {
        self.refresh_token_for_services(new_token)
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
    pub const fn ledger_identity_service(&self) -> &DamlLedgerIdentityService {
        &self.ledger_identity_service
    }

    /// DOCME
    pub const fn ledger_configuration_service(&self) -> &DamlLedgerConfigurationService {
        &self.ledger_configuration_service
    }

    /// DOCME
    pub const fn package_service(&self) -> &DamlPackageService {
        &self.package_service
    }

    /// DOCME
    pub const fn command_submission_service(&self) -> &DamlCommandSubmissionService {
        &self.command_submission_service
    }

    /// DOCME
    pub const fn command_completion_service(&self) -> &DamlCommandCompletionService {
        &self.command_completion_service
    }

    /// DOCME
    pub const fn command_service(&self) -> &DamlCommandService {
        &self.command_service
    }

    /// DOCME
    pub const fn transaction_service(&self) -> &DamlTransactionService {
        &self.transaction_service
    }

    /// DOCME
    pub const fn active_contract_service(&self) -> &DamlActiveContractsService {
        &self.active_contract_service
    }

    /// DOCME
    #[cfg(feature = "admin")]
    pub const fn package_management_service(&self) -> &DamlPackageManagementService {
        &self.package_management_service
    }

    /// DOCME
    #[cfg(feature = "admin")]
    pub const fn party_management_service(&self) -> &DamlPartyManagementService {
        &self.party_management_service
    }

    /// DOCME
    #[cfg(feature = "admin")]
    pub const fn config_management_service(&self) -> &DamlConfigManagementService {
        &self.config_management_service
    }

    /// DOCME
    #[cfg(feature = "sandbox")]
    pub const fn time_service(&self) -> &DamlTimeService {
        &self.time_service
    }

    async fn open_channel(config: &DamlGrpcClientConfig) -> DamlResult<Channel> {
        let mut channel = Self::make_channel(config).await;
        let start = Instant::now();
        while let Err(e) = channel {
            if start.elapsed() > config.timeout {
                return Err(DamlError::new_timeout_error(e));
            }
            channel = Self::make_channel(config).await;
        }
        channel
    }

    async fn make_channel(config: &DamlGrpcClientConfig) -> DamlResult<Channel> {
        let mut channel = Channel::from_shared(config.uri.to_owned())?;
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
        channel.connect().await.map_err(DamlError::from)
    }

    async fn query_ledger_identity(
        timeout: &Duration,
        ledger_identity_service: &DamlLedgerIdentityService,
    ) -> DamlResult<String> {
        let start = Instant::now();
        let mut ledger_identity: DamlResult<String> = ledger_identity_service.get_ledger_identity().await;
        while let Err(e) = ledger_identity {
            if let DamlError::GRPCPermissionError(_) = e {
                return Err(e);
            }
            if start.elapsed() > *timeout {
                return Err(DamlError::new_timeout_error(e));
            }
            ledger_identity = ledger_identity_service.get_ledger_identity().await;
        }
        ledger_identity
    }

    async fn make_client_from_channel(channel: &Channel, config: DamlGrpcClientConfig) -> DamlResult<Self> {
        let auth_token = config.auth_token.clone();
        let ledger_identity_service = DamlLedgerIdentityService::new(channel.clone(), config.auth_token.clone());
        let ledger_identity = Self::query_ledger_identity(&config.timeout, &ledger_identity_service).await?;
        Ok(Self {
            config,
            ledger_identity: ledger_identity.clone(),
            ledger_identity_service,
            ledger_configuration_service: DamlLedgerConfigurationService::new(
                channel.clone(),
                &ledger_identity,
                auth_token.clone(),
            ),
            package_service: DamlPackageService::new(channel.clone(), &ledger_identity, auth_token.clone()),
            command_submission_service: DamlCommandSubmissionService::new(
                channel.clone(),
                &ledger_identity,
                auth_token.clone(),
            ),
            transaction_service: DamlTransactionService::new(channel.clone(), &ledger_identity, auth_token.clone()),
            command_service: DamlCommandService::new(channel.clone(), &ledger_identity, auth_token.clone()),
            command_completion_service: DamlCommandCompletionService::new(
                channel.clone(),
                &ledger_identity,
                auth_token.clone(),
            ),
            active_contract_service: DamlActiveContractsService::new(
                channel.clone(),
                &ledger_identity,
                auth_token.clone(),
            ),
            #[cfg(feature = "admin")]
            package_management_service: DamlPackageManagementService::new(channel.clone(), auth_token.clone()),
            #[cfg(feature = "admin")]
            party_management_service: DamlPartyManagementService::new(channel.clone(), auth_token.clone()),
            #[cfg(feature = "admin")]
            config_management_service: DamlConfigManagementService::new(channel.clone(), auth_token.clone()),
            #[cfg(feature = "sandbox")]
            reset_service: DamlResetService::new(channel.clone(), &ledger_identity, auth_token.clone()),
            #[cfg(feature = "sandbox")]
            time_service: DamlTimeService::new(channel.clone(), &ledger_identity, auth_token.clone()),
        })
    }

    // TODO maybe we wrap the token in a Cell?
    #[allow(clippy::redundant_clone)]
    fn refresh_token_for_services(mut self, new_token: impl Into<String>) -> Self {
        let new_token = new_token.into();
        self.config.auth_token = Some(new_token.clone());
        self.ledger_identity_service.refresh_token(Some(new_token.clone()));
        self.ledger_configuration_service.refresh_token(Some(new_token.clone()));
        self.package_service.refresh_token(Some(new_token.clone()));
        self.command_submission_service.refresh_token(Some(new_token.clone()));
        self.command_completion_service.refresh_token(Some(new_token.clone()));
        self.command_service.refresh_token(Some(new_token.clone()));
        self.transaction_service.refresh_token(Some(new_token.clone()));
        self.active_contract_service.refresh_token(Some(new_token.clone()));
        #[cfg(feature = "admin")]
        self.package_management_service.refresh_token(Some(new_token.clone()));
        #[cfg(feature = "admin")]
        self.party_management_service.refresh_token(Some(new_token.clone()));
        #[cfg(feature = "admin")]
        self.config_management_service.refresh_token(Some(new_token.clone()));
        #[cfg(feature = "sandbox")]
        self.reset_service.refresh_token(Some(new_token.clone()));
        #[cfg(feature = "sandbox")]
        self.time_service.refresh_token(Some(new_token));
        self
    }
}

/// Refresh the token used by a ledger API service.
pub trait DamlTokenRefresh {
    fn refresh_token(&mut self, auth_token: Option<String>);
}
