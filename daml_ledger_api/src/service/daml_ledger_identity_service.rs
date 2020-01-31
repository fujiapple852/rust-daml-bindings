use crate::data::DamlResult;
use crate::data::{DamlError, DamlTraceContext};
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::ledger_identity_service_client::LedgerIdentityServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::{GetLedgerIdentityRequest, TraceContext};
use crate::ledger_client::DamlTokenRefresh;
use crate::service::common::make_request;
use log::{debug, trace};
use tonic::transport::Channel;

/// Obtain the unique identity that the DAML ledger.
pub struct DamlLedgerIdentityService {
    channel: Channel,
    auth_token: Option<String>,
}

impl DamlLedgerIdentityService {
    pub fn new(channel: Channel, auth_token: Option<String>) -> Self {
        Self {
            channel,
            auth_token,
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
        debug!("get_ledger_identity");
        let payload = GetLedgerIdentityRequest {
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!("get_ledger_identity payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let async_result = self
            .client()
            .get_ledger_identity(make_request(payload, &self.auth_token)?)
            .await
            .map_err(DamlError::from)?;
        Ok(async_result.into_inner().ledger_id)
    }

    fn client(&self) -> LedgerIdentityServiceClient<Channel> {
        LedgerIdentityServiceClient::new(self.channel.clone())
    }
}

impl DamlTokenRefresh for DamlLedgerIdentityService {
    fn refresh_token(&mut self, auth_token: Option<String>) {
        self.auth_token = auth_token
    }
}
