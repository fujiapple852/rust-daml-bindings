use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::daml::ledger::api::v1::admin::TimeModel;
use crate::util::{from_grpc_duration, to_grpc_duration, Required};
use std::convert::TryFrom;
use std::time::Duration;

/// The ledger time model.
#[derive(Debug, PartialEq, Eq, Default, Clone, Hash)]
pub struct DamlTimeModel {
    pub avg_transaction_latency: Duration,
    pub min_skew: Duration,
    pub max_skew: Duration,
}
impl DamlTimeModel {
    pub fn new(
        avg_transaction_latency: impl Into<Duration>,
        min_skew: impl Into<Duration>,
        max_skew: impl Into<Duration>,
    ) -> Self {
        Self {
            avg_transaction_latency: avg_transaction_latency.into(),
            min_skew: min_skew.into(),
            max_skew: max_skew.into(),
        }
    }

    /// The expected average latency of a transaction, i.e., the average time
    /// from submitting the transaction to a [`WriteService`] and the transaction
    /// being assigned a record time.
    pub fn avg_transaction_latency(&self) -> &Duration {
        &self.avg_transaction_latency
    }

    /// The minimum skew between ledger time and record time: `lt_TX` >= `rt_TX` - minSkew
    pub fn min_skew(&self) -> &Duration {
        &self.min_skew
    }

    /// The maximum skew between ledger time and record time: `lt_TX` <= `rt_TX` + maxSkew
    pub fn max_skew(&self) -> &Duration {
        &self.max_skew
    }
}

impl TryFrom<DamlTimeModel> for TimeModel {
    type Error = DamlError;

    fn try_from(time_model: DamlTimeModel) -> DamlResult<Self> {
        Ok(Self {
            avg_transaction_latency: Some(to_grpc_duration(time_model.avg_transaction_latency())?),
            min_skew: Some(to_grpc_duration(time_model.min_skew())?),
            max_skew: Some(to_grpc_duration(time_model.max_skew())?),
        })
    }
}

impl TryFrom<TimeModel> for DamlTimeModel {
    type Error = DamlError;

    fn try_from(time_model: TimeModel) -> DamlResult<Self> {
        let avg_transaction_latency = from_grpc_duration(&time_model.avg_transaction_latency.req()?);
        let min_skew = from_grpc_duration(&time_model.min_skew.req()?);
        let max_skew = from_grpc_duration(&time_model.max_skew.req()?);
        Ok(Self::new(avg_transaction_latency, min_skew, max_skew))
    }
}
