use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::admin::TimeModel;
use crate::util::{from_grpc_duration, to_grpc_duration, Required};
use std::convert::TryFrom;
use std::time::Duration;

/// The ledger time model.
#[derive(Debug, PartialEq, Eq, Default, Clone, Hash)]
pub struct DamlTimeModel {
    pub min_transaction_latency: Duration,
    pub max_clock_skew: Duration,
    pub max_ttl: Duration,
}
impl DamlTimeModel {
    pub fn new(
        min_transaction_latency: impl Into<Duration>,
        max_clock_skew: impl Into<Duration>,
        max_ttl: impl Into<Duration>,
    ) -> Self {
        Self {
            min_transaction_latency: min_transaction_latency.into(),
            max_clock_skew: max_clock_skew.into(),
            max_ttl: max_ttl.into(),
        }
    }

    /// The expected minimum latency of a transaction.
    pub fn min_transaction_latency(&self) -> &Duration {
        &self.min_transaction_latency
    }

    /// The maximum allowed clock skew between the ledger and clients.
    pub fn max_clock_skew(&self) -> &Duration {
        &self.max_clock_skew
    }

    /// The maximum allowed time to live for a transaction.
    ///
    /// Must be greater than the derived minimum time to live.
    pub fn max_ttl(&self) -> &Duration {
        &self.max_ttl
    }
}

impl TryFrom<DamlTimeModel> for TimeModel {
    type Error = DamlError;

    fn try_from(time_model: DamlTimeModel) -> DamlResult<Self> {
        Ok(Self {
            min_transaction_latency: Some(to_grpc_duration(time_model.min_transaction_latency())?),
            max_clock_skew: Some(to_grpc_duration(time_model.max_clock_skew())?),
            max_ttl: Some(to_grpc_duration(time_model.max_ttl())?),
        })
    }
}

impl TryFrom<TimeModel> for DamlTimeModel {
    type Error = DamlError;

    fn try_from(time_model: TimeModel) -> DamlResult<Self> {
        let min_transaction_latency = from_grpc_duration(&time_model.min_transaction_latency.req()?);
        let max_clock_skew = from_grpc_duration(&time_model.max_clock_skew.req()?);
        let max_ttl = from_grpc_duration(&time_model.max_ttl.req()?);
        Ok(Self::new(min_transaction_latency, max_clock_skew, max_ttl))
    }
}
