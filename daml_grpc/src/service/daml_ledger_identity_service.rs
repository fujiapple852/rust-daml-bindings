use crate::data::DamlResult;
use crate::data::{DamlError, DamlTraceContext};
use crate::grpc_protobuf::com::daml::ledger::api::v1::ledger_identity_service_client::LedgerIdentityServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{GetLedgerIdentityRequest, TraceContext};
use crate::service::common::make_request;
use log::{debug, trace};
use tonic::transport::Channel;

/// Obtain the unique identity that the DAML ledger.
pub struct DamlLedgerIdentityService<'a> {
    channel: Channel,
    auth_token: Option<&'a str>,
}

impl<'a> DamlLedgerIdentityService<'a> {
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
            .get_ledger_identity(make_request(payload, self.auth_token.as_deref())?)
            .await
            .map_err(DamlError::from)?;
        Ok(async_result.into_inner().ledger_id)
    }

    fn client(&self) -> LedgerIdentityServiceClient<Channel> {
        LedgerIdentityServiceClient::new(self.channel.clone())
    }
}
