use tonic::transport::Channel;
use tracing::{instrument, trace};

use crate::data::DamlResult;
use crate::grpc_protobuf::com::daml::ledger::api::v1::testing::reset_service_client::ResetServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::testing::ResetRequest;
use crate::service::common::make_request;

/// Reset the state of a Daml ledger (requires `testing` feature).
#[derive(Debug)]
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

    #[instrument(skip(self))]
    pub async fn reset(&self) -> DamlResult<()> {
        let payload = ResetRequest {
            ledger_id: self.ledger_id.to_string(),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        self.client().reset(make_request(payload, self.auth_token)?).await?;
        Ok(())
    }

    fn client(&self) -> ResetServiceClient<Channel> {
        ResetServiceClient::new(self.channel.clone())
    }
}
