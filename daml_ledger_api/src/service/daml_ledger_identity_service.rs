use futures::Future;
use grpcio::Channel;
use grpcio::ClientUnaryReceiver;

use crate::data::DamlError;
use crate::data::DamlResult;
use crate::grpc_protobuf_autogen::ledger_identity_service::GetLedgerIdentityRequest;
use crate::grpc_protobuf_autogen::ledger_identity_service::GetLedgerIdentityResponse;
use crate::grpc_protobuf_autogen::ledger_identity_service_grpc::LedgerIdentityServiceClient;

/// Obtain the unique identity that the DAML ledger.
pub struct DamlLedgerIdentityService {
    grpc_client: LedgerIdentityServiceClient,
}

impl DamlLedgerIdentityService {
    pub fn new(channel: Channel) -> Self {
        Self {
            grpc_client: LedgerIdentityServiceClient::new(channel),
        }
    }

    pub fn get_ledger_identity(&self) -> DamlResult<impl Future<Item = String, Error = DamlError>> {
        let async_result: ClientUnaryReceiver<GetLedgerIdentityResponse> =
            self.grpc_client.get_ledger_identity_async(&GetLedgerIdentityRequest::new())?;
        Ok(async_result.map_err(Into::into).map(|mut r| r.take_ledger_id()))
    }

    pub fn get_ledger_identity_sync(&self) -> DamlResult<String> {
        self.get_ledger_identity()?.wait()
    }
}
