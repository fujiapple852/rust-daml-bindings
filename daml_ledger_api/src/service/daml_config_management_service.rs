use crate::data::DamlResult;
use crate::data::DamlTimeModel;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::admin::config_management_service_client::ConfigManagementServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::admin::{
    GetTimeModelRequest, SetTimeModelRequest, TimeModel,
};
use crate::util::{to_grpc_timestamp, Required};
use chrono::{DateTime, Utc};
use std::convert::TryFrom;
use tonic::transport::Channel;
use tonic::Request;

/// Provides methods for the ledger administrator to change the current ledger configuration.
///
/// The services provides methods to modify different aspects of the configuration.
pub struct DamlConfigManagementService {
    channel: Channel,
}

impl DamlConfigManagementService {
    pub fn new(channel: Channel) -> Self {
        Self {
            channel,
        }
    }

    /// Return the currently active time model and the current configuration generation.
    ///
    /// The current configuration generation. The generation is a monotonically increasing integer that is incremented
    /// on each change. Used when setting the time model.
    pub async fn get_time_model(&self) -> DamlResult<(i64, DamlTimeModel)> {
        let request = Request::new(GetTimeModelRequest {});
        let response = self.client().get_time_model(request).await?.into_inner();
        Ok((response.configuration_generation, DamlTimeModel::try_from(response.time_model.req()?)?))
    }

    /// Set the ledger time model.
    ///
    /// # Errors
    ///
    /// In case of failure this method responds with:
    ///
    /// `INVALID_ARGUMENT` if arguments are invalid, or the provided configuration generation
    ///   does not match the current active configuration generation. The caller is expected
    ///   to retry by again fetching current time model using `GetTimeModel`, applying changes
    ///   and resubmitting.
    ///
    /// `ABORTED` if the request is rejected or times out. Note that a timed out request may
    ///   have still been committed to the ledger. Application should re-query the current
    ///   time model before retrying.
    ///
    /// `UNIMPLEMENTED` if this method is not supported by the backing ledger.
    pub async fn set_time_model(
        &self,
        submission_id: impl Into<String>,
        maximum_record_time: impl Into<DateTime<Utc>>,
        configuration_generation: i64,
        new_time_model: impl Into<DamlTimeModel>,
    ) -> DamlResult<i64> {
        let request = Request::new(SetTimeModelRequest {
            submission_id: submission_id.into(),
            maximum_record_time: Some(to_grpc_timestamp(maximum_record_time.into())?),
            configuration_generation,
            new_time_model: Some(TimeModel::try_from(new_time_model.into())?),
        });
        let response = self.client().set_time_model(request).await?.into_inner();
        Ok(response.configuration_generation)
    }

    fn client(&self) -> ConfigManagementServiceClient<Channel> {
        ConfigManagementServiceClient::new(self.channel.clone())
    }
}
