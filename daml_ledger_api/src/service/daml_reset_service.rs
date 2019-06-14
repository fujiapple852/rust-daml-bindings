use futures::Future;
use grpcio::Channel;
use grpcio::ClientUnaryReceiver;

use crate::data::DamlError;
use crate::data::DamlResult;
use crate::grpc_protobuf_autogen::empty::Empty;
use crate::grpc_protobuf_autogen::reset_service::ResetRequest;
use crate::grpc_protobuf_autogen::reset_service_grpc::ResetServiceClient;

/// Reset the state of a DAML ledger (requires `testing` feature).
pub struct DamlResetService {
    grpc_client: ResetServiceClient,
    ledger_id: String,
}

impl DamlResetService {
    pub fn new(channel: Channel, ledger_id: String) -> Self {
        Self {
            grpc_client: ResetServiceClient::new(channel),
            ledger_id,
        }
    }

    pub fn reset(&self) -> DamlResult<impl Future<Item = (), Error = DamlError>> {
        let mut request = ResetRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        let async_result: ClientUnaryReceiver<Empty> = self.grpc_client.reset_async(&request)?;
        Ok(async_result.map_err(Into::into).map(|_| ()))
    }

    pub fn reset_sync(&self) -> DamlResult<()> {
        self.reset()?.wait()
    }
}
