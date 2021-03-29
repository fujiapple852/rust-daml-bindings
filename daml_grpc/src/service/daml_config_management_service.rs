use crate::data::DamlResult;
use crate::data::DamlTimeModel;
use crate::grpc_protobuf::com::daml::ledger::api::v1::admin::config_management_service_client::ConfigManagementServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::admin::{GetTimeModelRequest, SetTimeModelRequest, TimeModel};
use crate::service::common::make_request;
use crate::util::{to_grpc_timestamp, Required};
use chrono::{DateTime, Utc};
use std::convert::TryFrom;
use std::fmt::Debug;
use tonic::transport::Channel;
use tracing::{instrument, trace};

/// Provides methods for the ledger administrator to change the current ledger configuration.
///
/// The services provides methods to modify different aspects of the configuration.
#[derive(Debug)]
pub struct DamlConfigManagementService<'a> {
    channel: Channel,
    auth_token: Option<&'a str>,
}

impl<'a> DamlConfigManagementService<'a> {
    pub fn new(channel: Channel, auth_token: Option<&'a str>) -> Self {
        Self {
            channel,
            auth_token,
        }
    }

    /// Override the JWT token to use for this service.
    pub fn with_token(self, auth_token: &'a str) -> Self {
        Self {
            auth_token: Some(auth_token),
            ..self
        }
    }

    /// Return the currently active time model and the current configuration generation.
    ///
    /// The current configuration generation. The generation is a monotonically increasing integer that is incremented
    /// on each change. Used when setting the time model.
    #[instrument(skip(self))]
    pub async fn get_time_model(&self) -> DamlResult<(i64, DamlTimeModel)> {
        let payload = GetTimeModelRequest {};
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().get_time_model(make_request(payload, self.auth_token.as_deref())?).await?.into_inner();
        trace!(?response);
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
    #[instrument(skip(self))]
    pub async fn set_time_model(
        &self,
        submission_id: impl Into<String> + Debug,
        maximum_record_time: impl Into<DateTime<Utc>> + Debug,
        configuration_generation: i64,
        new_time_model: impl Into<DamlTimeModel> + Debug,
    ) -> DamlResult<i64> {
        let payload = SetTimeModelRequest {
            submission_id: submission_id.into(),
            maximum_record_time: Some(to_grpc_timestamp(maximum_record_time.into())?),
            configuration_generation,
            new_time_model: Some(TimeModel::try_from(new_time_model.into())?),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self.client().set_time_model(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        Ok(response.configuration_generation)
    }

    fn client(&self) -> ConfigManagementServiceClient<Channel> {
        ConfigManagementServiceClient::new(self.channel.clone())
    }
}
