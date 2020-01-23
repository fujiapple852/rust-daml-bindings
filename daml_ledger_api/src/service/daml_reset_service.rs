use crate::data::DamlResult;

use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::testing::reset_service_client::ResetServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::testing::ResetRequest;
use tonic::transport::Channel;
use tonic::Request;

/// Reset the state of a DAML ledger (requires `testing` feature).
pub struct DamlResetService {
    channel: Channel,
    ledger_id: String,
}

impl DamlResetService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
        }
    }

    pub async fn reset(&self) -> DamlResult<()> {
        let request = Request::new(ResetRequest {
            ledger_id: self.ledger_id.clone(),
        });
        self.client().reset(request).await?;
        Ok(())
    }

    fn client(&self) -> ResetServiceClient<Channel> {
        ResetServiceClient::new(self.channel.clone())
    }
}
