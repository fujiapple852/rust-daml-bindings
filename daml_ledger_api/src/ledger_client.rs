use grpcio::{Channel, ChannelBuilder, EnvBuilder};
use std::sync::Arc;
#[cfg(feature = "testing")]
use std::time::{Duration, Instant};

#[cfg(feature = "testing")]
use crate::data::DamlError::ResetTimeout;
use crate::data::DamlResult;
use crate::service::DamlActiveContractsService;
use crate::service::DamlCommandCompletionService;
use crate::service::DamlCommandService;
use crate::service::DamlCommandSubmissionService;
use crate::service::DamlLedgerConfigurationService;
use crate::service::DamlLedgerIdentityService;
use crate::service::DamlPackageService;
#[cfg(feature = "testing")]
use crate::service::DamlResetService;
#[cfg(feature = "testing")]
use crate::service::DamlTimeService;
use crate::service::DamlTransactionService;
use log::info;

#[cfg(feature = "testing")]
const RESET_TIMEOUT: Duration = Duration::from_secs(5);

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
    #[cfg(feature = "testing")]
    reset_service: DamlResetService,
    #[cfg(feature = "testing")]
    time_service: DamlTimeService,
}

impl DamlLedgerClient {
    pub fn connect(hostname: impl Into<String>, port: u16) -> DamlResult<Self> {
        let hostname = hostname.into();
        Self::make_client_from_channel(&Self::make_channel(&hostname, port), hostname, port)
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

    #[cfg(feature = "testing")]
    pub fn time_service(&self) -> &DamlTimeService {
        &self.time_service
    }

    #[cfg(feature = "testing")]
    pub fn reset_and_wait(self) -> DamlResult<Self> {
        self.reset_service.reset_sync()?;
        let channel = Self::make_channel(&self.hostname, self.port);
        let ledger_identity_service = DamlLedgerIdentityService::new(channel.clone());
        let start = Instant::now();
        let mut ledger_identity: DamlResult<String> = ledger_identity_service.get_ledger_identity_sync();
        while let Err(_) = ledger_identity {
            if start.elapsed() > RESET_TIMEOUT {
                return Err(ResetTimeout);
            }
            ledger_identity = ledger_identity_service.get_ledger_identity_sync();
        }
        Self::make_client_from_channel(&channel, self.hostname, self.port)
    }

    fn make_channel(hostname: &str, port: u16) -> Channel {
        let env = Arc::new(EnvBuilder::new().build());
        ChannelBuilder::new(env).connect(&format!("{}:{}", hostname, port))
    }

    fn make_client_from_channel(channel: &Channel, hostname: String, port: u16) -> DamlResult<Self> {
        info!("making client for {}:{}", hostname, port);
        let ledger_identity_service = DamlLedgerIdentityService::new(channel.clone());
        let ledger_identity = ledger_identity_service.get_ledger_identity_sync()?;
        Ok(Self {
            hostname,
            port,
            ledger_identity: ledger_identity.clone(),
            ledger_identity_service,
            ledger_configuration_service: DamlLedgerConfigurationService::new(channel.clone(), ledger_identity.clone()),
            package_service: DamlPackageService::new(channel.clone(), ledger_identity.clone()),
            command_submission_service: DamlCommandSubmissionService::new(channel.clone(), ledger_identity.clone()),
            transaction_service: DamlTransactionService::new(channel.clone(), ledger_identity.clone()),
            command_service: DamlCommandService::new(channel.clone(), ledger_identity.clone()),
            command_completion_service: DamlCommandCompletionService::new(channel.clone(), ledger_identity.clone()),
            active_contract_service: DamlActiveContractsService::new(channel.clone(), ledger_identity.clone()),
            #[cfg(feature = "testing")]
            reset_service: DamlResetService::new(channel.clone(), ledger_identity.clone()),
            #[cfg(feature = "testing")]
            time_service: DamlTimeService::new(channel.clone(), ledger_identity.clone()),
        })
    }
}
