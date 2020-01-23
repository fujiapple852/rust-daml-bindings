use crate::data::DamlResult;
use crate::data::{DamlError, DamlTraceContext};
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::ledger_identity_service_client::LedgerIdentityServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::{GetLedgerIdentityRequest, TraceContext};
use tonic::transport::Channel;
use tonic::Request;

/// Obtain the unique identity that the DAML ledger.
pub struct DamlLedgerIdentityService {
    channel: Channel,
}

impl DamlLedgerIdentityService {
    pub fn new(channel: Channel) -> Self {
        Self {
            channel,
        }
    }

    /// DOCME fully document this.
    pub async fn get_ledger_identity(&self) -> DamlResult<String> {
        self.get_ledger_identity_with_trace(None).await
    }

    pub async fn get_ledger_identity_with_trace(
        &self,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<String> {
        let request = Request::new(GetLedgerIdentityRequest {
            trace_context: trace_context.into().map(TraceContext::from),
        });
        let async_result = self.client().get_ledger_identity(request).await.map_err(DamlError::from)?;
        Ok(async_result.into_inner().ledger_id)
    }

    fn client(&self) -> LedgerIdentityServiceClient<Channel> {
        LedgerIdentityServiceClient::new(self.channel.clone())
    }
}
