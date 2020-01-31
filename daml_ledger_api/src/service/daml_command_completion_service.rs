use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;

use crate::data::completion::DamlCompletionResponse;
use crate::data::offset::DamlLedgerOffset;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::command_completion_service_client::CommandCompletionServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::{
    CompletionEndRequest, CompletionStreamRequest, LedgerOffset, TraceContext,
};
use crate::util::Required;

use crate::ledger_client::DamlTokenRefresh;
use crate::service::common::make_request;
use futures::Stream;
use futures::StreamExt;
use log::{debug, trace};
use std::convert::TryFrom;
use tonic::transport::Channel;

/// Observe the status of command submissions on a DAML ledger.
pub struct DamlCommandCompletionService {
    channel: Channel,
    ledger_id: String,
    auth_token: Option<String>,
}

impl DamlCommandCompletionService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
            auth_token,
        }
    }

    /// DOCME fully document this
    pub async fn get_completion_stream(
        &self,
        application_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        offset: impl Into<DamlLedgerOffset>,
    ) -> DamlResult<impl Stream<Item = DamlResult<DamlCompletionResponse>>> {
        debug!("get_completion_stream");
        let payload = CompletionStreamRequest {
            ledger_id: self.ledger_id.clone(),
            application_id: application_id.into(),
            offset: Some(LedgerOffset::from(offset.into())),
            parties: parties.into(),
        };
        trace!("get_completion_stream payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let completion_stream =
            self.client().completion_stream(make_request(payload, &self.auth_token)?).await?.into_inner();
        Ok(completion_stream.map(|item| match item {
            Ok(completion) => DamlCompletionResponse::try_from(completion),
            Err(e) => Err(DamlError::from(e)),
        }))
    }

    /// DOCME fully document this
    pub async fn get_completion_end(&self) -> DamlResult<DamlLedgerOffset> {
        self.get_completion_end_with_trace(None).await
    }

    pub async fn get_completion_end_with_trace(
        &self,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<DamlLedgerOffset> {
        debug!("get_completion_end");
        let payload = CompletionEndRequest {
            ledger_id: self.ledger_id.clone(),
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!("get_completion_end payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        self.client()
            .completion_end(make_request(payload, &self.auth_token)?)
            .await
            .map_err(DamlError::from)?
            .into_inner()
            .offset
            .req()
            .map(DamlLedgerOffset::try_from)?
    }

    fn client(&self) -> CommandCompletionServiceClient<Channel> {
        CommandCompletionServiceClient::new(self.channel.clone())
    }
}

impl DamlTokenRefresh for DamlCommandCompletionService {
    fn refresh_token(&mut self, auth_token: Option<String>) {
        self.auth_token = auth_token
    }
}
