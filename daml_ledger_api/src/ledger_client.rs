use crate::data::{DamlError, DamlResult};
use crate::service::{
    DamlActiveContractsService, DamlCommandCompletionService, DamlCommandService, DamlCommandSubmissionService,
    DamlLedgerConfigurationService, DamlLedgerIdentityService, DamlPackageService, DamlTransactionService,
};
#[cfg(feature = "admin")]
use crate::service::{DamlConfigManagementService, DamlPackageManagementService, DamlPartyManagementService};
#[cfg(feature = "testing")]
use crate::service::{DamlResetService, DamlTimeService};
use log::debug;
use std::time::{Duration, Instant};
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

const DEFAULT_TIMEOUT_SECS: u64 = 5;

#[derive(Debug, Default)]
pub struct ChannelConfig {
    uri: String,
    timeout: Duration,
    concurrency_limit: Option<usize>,
    rate_limit: Option<(u64, Duration)>,
    initial_stream_window_size: Option<u32>,
    initial_connection_window_size: Option<u32>,
    tcp_keepalive: Option<Duration>,
    tcp_nodelay: bool,
    tls_config: Option<DamlTlsConfig>,
    auth_token: Option<String>,
}

#[derive(Debug)]
pub struct DamlTlsConfig {
    ca_cert: Option<Vec<u8>>,
}

pub struct DamlLedgerClientBuilder {
    config: ChannelConfig,
}

impl DamlLedgerClientBuilder {
    pub fn uri(uri: impl Into<String>) -> Self {
        Self {
            config: ChannelConfig {
                uri: uri.into(),
                timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
                ..ChannelConfig::default()
            },
        }
    }

    pub fn timeout(self, timeout: Duration) -> Self {
        Self {
            config: ChannelConfig {
                timeout,
                ..self.config
            },
        }
    }

    pub fn with_tls(self, ca_cert: impl Into<Vec<u8>>) -> Self {
        Self {
            config: ChannelConfig {
                tls_config: Some(DamlTlsConfig {
                    ca_cert: Some(ca_cert.into()),
                }),
                ..self.config
            },
        }
    }

    pub fn with_auth(self, auth_token: String) -> Self {
        Self {
            config: ChannelConfig {
                auth_token: Some(auth_token),
                ..self.config
            },
        }
    }

    pub async fn connect(self) -> DamlResult<DamlLedgerClient> {
        DamlLedgerClient::connect(self.config).await
    }
}

/// DAML ledger client connection.
pub struct DamlLedgerClient {
    config: ChannelConfig,
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
    #[cfg(feature = "testing")]
    reset_service: DamlResetService,
    #[cfg(feature = "testing")]
    time_service: DamlTimeService,
}

impl DamlLedgerClient {
    /// Create a channel and connect.
    pub async fn connect(config: ChannelConfig) -> DamlResult<Self> {
        debug!("connecting to {}", config.uri);
        let channel = Self::open_channel(&config).await?;
        Self::make_client_from_channel(&channel, config).await
    }

    /// Reset the ledger and reconnect.
    #[cfg(feature = "testing")]
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
    pub fn channel_config(&self) -> &ChannelConfig {
        &self.config
    }

    pub fn ledger_identity(&self) -> &str {
        &self.ledger_identity
    }

    pub fn ledger_identity_service(&self) -> &DamlLedgerIdentityService {
        &self.ledger_identity_service
    }

    pub fn ledger_configuration_service(&self) -> &DamlLedgerConfigurationService {
        &self.ledger_configuration_service
    }

    pub fn package_service(&self) -> &DamlPackageService {
        &self.package_service
    }

    pub fn command_submission_service(&self) -> &DamlCommandSubmissionService {
        &self.command_submission_service
    }

    pub fn command_completion_service(&self) -> &DamlCommandCompletionService {
        &self.command_completion_service
    }

    pub fn command_service(&self) -> &DamlCommandService {
        &self.command_service
    }

    pub fn transaction_service(&self) -> &DamlTransactionService {
        &self.transaction_service
    }

    pub fn active_contract_service(&self) -> &DamlActiveContractsService {
        &self.active_contract_service
    }

    #[cfg(feature = "admin")]
    pub fn package_management_service(&self) -> &DamlPackageManagementService {
        &self.package_management_service
    }

    #[cfg(feature = "admin")]
    pub fn party_management_service(&self) -> &DamlPartyManagementService {
        &self.party_management_service
    }

    #[cfg(feature = "admin")]
    pub fn config_management_service(&self) -> &DamlConfigManagementService {
        &self.config_management_service
    }

    #[cfg(feature = "testing")]
    pub fn time_service(&self) -> &DamlTimeService {
        &self.time_service
    }

    async fn open_channel(config: &ChannelConfig) -> DamlResult<Channel> {
        let mut channel = Self::make_channel(config).await;
        let start = Instant::now();
        while let Err(e) = channel {
            if start.elapsed() > config.timeout {
                return Err(e);
            }
            channel = Self::make_channel(config).await;
        }
        channel
    }

    async fn make_channel(config: &ChannelConfig) -> DamlResult<Channel> {
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
            Some(DamlTlsConfig {
                ca_cert: Some(cert),
            }) => {
                channel = channel.tls_config(ClientTlsConfig::new().ca_certificate(Certificate::from_pem(cert)));
            },
            Some(DamlTlsConfig {
                ca_cert: None,
            }) => {
                channel = channel.tls_config(ClientTlsConfig::new());
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
        if let e @ Err(DamlError::GRPCPermissionError(_)) = ledger_identity {
            return e;
        }
        while let Err(e) = ledger_identity {
            if start.elapsed() > *timeout {
                return Err(e);
            }
            ledger_identity = ledger_identity_service.get_ledger_identity().await;
        }
        ledger_identity
    }

    async fn make_client_from_channel(channel: &Channel, config: ChannelConfig) -> DamlResult<Self> {
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
            #[cfg(feature = "testing")]
            reset_service: DamlResetService::new(channel.clone(), &ledger_identity, auth_token.clone()),
            #[cfg(feature = "testing")]
            time_service: DamlTimeService::new(channel.clone(), &ledger_identity, auth_token.clone()),
        })
    }

    // TODO maybe we wrap the token in a Cell?
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
        #[cfg(feature = "testing")]
        self.reset_service.refresh_token(Some(new_token.clone()));
        #[cfg(feature = "testing")]
        self.time_service.refresh_token(Some(new_token));
        self
    }
}

/// Refresh the token used by a ledger API service.
pub trait DamlTokenRefresh {
    fn refresh_token(&mut self, auth_token: Option<String>);
}
