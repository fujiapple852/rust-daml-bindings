use crate::data::DamlResult;
use crate::grpc_protobuf::com::daml::ledger::api::v1::testing::reset_service_client::ResetServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::testing::ResetRequest;
use crate::service::common::make_request;
use log::{debug, trace};
use tonic::transport::Channel;

/// Reset the state of a DAML ledger (requires `testing` feature).
pub struct DamlResetService<'a> {
    channel: Channel,
    ledger_id: &'a str,
    auth_token: Option<&'a str>,
}

impl<'a> DamlResetService<'a> {
    pub fn new(channel: Channel, ledger_id: &'a str, auth_token: Option<&'a str>) -> Self {
        Self {
            channel,
            ledger_id,
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

    /// Override the ledger id to use for this service.
    pub fn with_ledger_id(self, ledger_id: &'a str) -> Self {
        Self {
            ledger_id,
            ..self
        }
    }

    pub async fn reset(&self) -> DamlResult<()> {
        debug!("reset");
        let payload = ResetRequest {
            ledger_id: self.ledger_id.to_string(),
        };
        trace!("reset payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        self.client().reset(make_request(payload, self.auth_token.as_deref())?).await?;
        Ok(())
    }

    fn client(&self) -> ResetServiceClient<Channel> {
        ResetServiceClient::new(self.channel.clone())
    }
}
