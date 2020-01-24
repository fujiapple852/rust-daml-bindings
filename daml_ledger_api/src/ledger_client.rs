use crate::data::{DamlError, DamlResult};
use crate::service::{
    DamlActiveContractsService, DamlCommandCompletionService, DamlCommandService, DamlCommandSubmissionService,
    DamlLedgerConfigurationService, DamlLedgerIdentityService, DamlPackageService, DamlTransactionService,
};
#[cfg(feature = "admin")]
use crate::service::{DamlConfigManagementService, DamlPackageManagementService, DamlPartyManagementService};
#[cfg(feature = "testing")]
use crate::service::{DamlResetService, DamlTimeService};
use std::time::{Duration, Instant};
use tonic::transport::Channel;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// DAML ledger client connection.
pub struct DamlLedgerClient {
    hostname: String,
    port: u16,
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
    pub async fn connect(hostname: impl Into<String>, port: u16) -> DamlResult<Self> {
        let hostname = hostname.into();
        let channel = Self::wait_for_channel(&hostname, port).await?;
        let ledger_identity = Self::wait_for_ledger_identity(&channel).await?;
        Self::make_client_from_channel(&channel, &ledger_identity, hostname, port).await
    }

    #[cfg(feature = "testing")]
    pub async fn reset_and_wait(self) -> DamlResult<Self> {
        self.reset_service.reset().await?;
        let channel = Self::wait_for_channel(&self.hostname, self.port).await?;
        let ledger_identity = Self::wait_for_ledger_identity(&channel).await?;
        Self::make_client_from_channel(&channel, &ledger_identity, self.hostname, self.port).await
    }

    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    pub fn port(&self) -> u16 {
        self.port
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

    async fn wait_for_channel(hostname: &str, port: u16) -> DamlResult<Channel> {
        let mut channel = Self::open_channel(hostname, port).await;
        let start = Instant::now();
        while let Err(e) = channel {
            if start.elapsed() > DEFAULT_TIMEOUT {
                return Err(e);
            }
            channel = Self::open_channel(hostname, port).await;
        }
        channel
    }

    async fn wait_for_ledger_identity(channel: &Channel) -> DamlResult<String> {
        let ledger_identity_service = DamlLedgerIdentityService::new(channel.clone());
        let start = Instant::now();
        let mut ledger_identity: DamlResult<String> = ledger_identity_service.get_ledger_identity().await;
        while let Err(e) = ledger_identity {
            if start.elapsed() > DEFAULT_TIMEOUT {
                return Err(e);
            }
            ledger_identity = ledger_identity_service.get_ledger_identity().await;
        }
        ledger_identity
    }

    // TODO better control of connection string for clients needed
    // TODO support tls and other options, perhaps let users BYO channel?
    async fn open_channel(hostname: &str, port: u16) -> DamlResult<Channel> {
        Channel::from_shared(format!("http://{}:{}", hostname, port))?.connect().await.map_err(DamlError::from)
    }

    async fn make_client_from_channel(
        channel: &Channel,
        ledger_identity: &str,
        hostname: String,
        port: u16,
    ) -> DamlResult<Self> {
        Ok(Self {
            hostname,
            port,
            ledger_identity: ledger_identity.to_owned(),
            ledger_identity_service: DamlLedgerIdentityService::new(channel.clone()),
            ledger_configuration_service: DamlLedgerConfigurationService::new(channel.clone(), ledger_identity),
            package_service: DamlPackageService::new(channel.clone(), ledger_identity),
            command_submission_service: DamlCommandSubmissionService::new(channel.clone(), ledger_identity),
            transaction_service: DamlTransactionService::new(channel.clone(), ledger_identity),
            command_service: DamlCommandService::new(channel.clone(), ledger_identity),
            command_completion_service: DamlCommandCompletionService::new(channel.clone(), ledger_identity),
            active_contract_service: DamlActiveContractsService::new(channel.clone(), ledger_identity),
            #[cfg(feature = "admin")]
            package_management_service: DamlPackageManagementService::new(channel.clone()),
            #[cfg(feature = "admin")]
            party_management_service: DamlPartyManagementService::new(channel.clone()),
            #[cfg(feature = "admin")]
            config_management_service: DamlConfigManagementService::new(channel.clone()),
            #[cfg(feature = "testing")]
            reset_service: DamlResetService::new(channel.clone(), ledger_identity),
            #[cfg(feature = "testing")]
            time_service: DamlTimeService::new(channel.clone(), ledger_identity),
        })
    }
}
