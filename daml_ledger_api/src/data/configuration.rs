use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::LedgerConfiguration;
use crate::util::Required;
use std::convert::TryFrom;
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

impl TryFrom<LedgerConfiguration> for DamlLedgerConfiguration {
    type Error = DamlError;

    fn try_from(response: LedgerConfiguration) -> DamlResult<Self> {
        let min: prost_types::Duration = response.min_ttl.req()?;
        let max: prost_types::Duration = response.max_ttl.req()?;
        Ok(Self::new(from_duration(&min), from_duration(&max)))
    }
}

#[allow(clippy::cast_sign_loss)]
fn from_duration(duration: &prost_types::Duration) -> Duration {
    Duration::new(duration.seconds as u64, duration.nanos as u32)
}
