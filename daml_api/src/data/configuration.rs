use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::daml::ledger::api::v1::LedgerConfiguration;
use crate::util::{from_grpc_duration, Required};
use std::convert::TryFrom;
use std::time::Duration;

/// DAML ledger configuration information.
#[derive(Debug, Eq, PartialEq, Default)]
pub struct DamlLedgerConfiguration {
    pub max_deduplication_time: Duration,
}

impl DamlLedgerConfiguration {
    pub fn new(max_deduplication_time: Duration) -> Self {
        Self {
            max_deduplication_time,
        }
    }

    pub fn max_deduplication_time(&self) -> &Duration {
        &self.max_deduplication_time
    }
}

impl TryFrom<LedgerConfiguration> for DamlLedgerConfiguration {
    type Error = DamlError;

    fn try_from(response: LedgerConfiguration) -> DamlResult<Self> {
        let max_deduplication_time: prost_types::Duration = response.max_deduplication_time.req()?;
        Ok(Self::new(from_grpc_duration(&max_deduplication_time)))
    }
}
