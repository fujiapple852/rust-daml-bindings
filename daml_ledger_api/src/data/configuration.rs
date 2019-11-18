use crate::grpc_protobuf_autogen::ledger_configuration_service::LedgerConfiguration;
use protobuf::well_known_types;
use std::time::Duration;

/// DAML ledger configuration information.
#[derive(Debug, Eq, PartialEq, Default)]
pub struct DamlLedgerConfiguration {
    pub min_ttl: Duration,
    pub max_ttl: Duration,
}

impl DamlLedgerConfiguration {
    pub fn new(min_ttl: Duration, max_ttl: Duration) -> Self {
        Self {
            min_ttl,
            max_ttl,
        }
    }

    pub fn min_ttl(&self) -> &Duration {
        &self.min_ttl
    }

    pub fn max_ttl(&self) -> &Duration {
        &self.max_ttl
    }
}

impl From<LedgerConfiguration> for DamlLedgerConfiguration {
    fn from(mut response: LedgerConfiguration) -> Self {
        let min: well_known_types::Duration = response.take_min_ttl();
        let max: well_known_types::Duration = response.take_max_ttl();
        Self::new(from_duration(&min), from_duration(&max))
    }
}

#[allow(clippy::cast_sign_loss)]
fn from_duration(duration: &well_known_types::Duration) -> Duration {
    Duration::new(duration.get_seconds() as u64, duration.get_nanos() as u32)
}
