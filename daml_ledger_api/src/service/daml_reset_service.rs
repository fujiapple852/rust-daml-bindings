use crate::data::DamlResult;

use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::testing::reset_service_client::ResetServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::testing::ResetRequest;
use crate::ledger_client::DamlTokenRefresh;
use crate::service::common::make_request;
use log::{debug, trace};
use tonic::transport::Channel;

/// Reset the state of a DAML ledger (requires `testing` feature).
pub struct DamlResetService {
    channel: Channel,
    ledger_id: String,
    auth_token: Option<String>,
}

impl DamlResetService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
            auth_token,
        }
    }

    pub async fn reset(&self) -> DamlResult<()> {
        debug!("reset");
        let payload = ResetRequest {
            ledger_id: self.ledger_id.clone(),
        };
        trace!("reset payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        self.client().reset(make_request(payload, &self.auth_token)?).await?;
        Ok(())
    }

    fn client(&self) -> ResetServiceClient<Channel> {
        ResetServiceClient::new(self.channel.clone())
    }
}

impl DamlTokenRefresh for DamlResetService {
    fn refresh_token(&mut self, auth_token: Option<String>) {
        self.auth_token = auth_token
    }
}
