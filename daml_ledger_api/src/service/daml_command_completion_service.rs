use futures::Future;
use grpcio::Channel;
use grpcio::ClientUnaryReceiver;

use crate::data::completion::DamlCompletionResponse;
use crate::data::offset::DamlLedgerOffset;
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::grpc_protobuf_autogen::command_completion_service::CompletionEndRequest;
use crate::grpc_protobuf_autogen::command_completion_service::CompletionEndResponse;
use crate::grpc_protobuf_autogen::command_completion_service::CompletionStreamRequest;
use crate::grpc_protobuf_autogen::command_completion_service::CompletionStreamResponse;
use crate::grpc_protobuf_autogen::command_completion_service_grpc::CommandCompletionServiceClient;
use futures::future::{err, ok};
use futures::Stream;
use grpcio::ClientSStreamReceiver;
use std::convert::TryInto;

/// Observe the status of command submissions on a DAML ledger.
pub struct DamlCommandCompletionService {
    grpc_client: CommandCompletionServiceClient,
    ledger_id: String,
}

impl DamlCommandCompletionService {
    pub fn new(channel: Channel, ledger_id: String) -> Self {
        Self {
            grpc_client: CommandCompletionServiceClient::new(channel),
            ledger_id,
        }
    }

    pub fn get_completion_stream(
        &self,
        application_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        offset: impl Into<DamlLedgerOffset>,
    ) -> DamlResult<impl Stream<Item = DamlCompletionResponse, Error = DamlError>> {
        let mut request = CompletionStreamRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        request.set_application_id(application_id.into());
        request.set_offset(offset.into().into());
        request.set_parties(parties.into().into());
        let async_response: ClientSStreamReceiver<CompletionStreamResponse> =
            self.grpc_client.completion_stream(&request)?;
        Ok(async_response.map_err(Into::into).map(TryInto::try_into).and_then(|completion| match completion {
            Ok(c) => ok(c),
            Err(e) => err(e),
        }))
    }

    /// TODO fully document this
    pub fn get_completion_end(&self) -> DamlResult<impl Future<Item = DamlLedgerOffset, Error = DamlError>> {
        self.get_completion_end_with_trace(None)
    }

    pub fn get_completion_end_sync(&self) -> DamlResult<DamlLedgerOffset> {
        self.get_completion_end_with_trace(None)?.wait()
    }

    pub fn get_completion_end_with_trace_sync(
        &self,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<DamlLedgerOffset> {
        self.get_completion_end_with_trace(trace_context)?.wait()
    }

    pub fn get_completion_end_with_trace(
        &self,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Future<Item = DamlLedgerOffset, Error = DamlError>> {
        let mut request = CompletionEndRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        if let Some(tc) = trace_context.into() {
            request.set_trace_context(tc.into());
        }
        let async_response: ClientUnaryReceiver<CompletionEndResponse> =
            self.grpc_client.completion_end_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|mut r| r.take_offset().try_into()).and_then(|completion| {
            match completion {
                Ok(c) => ok(c),
                Err(e) => err(e),
            }
        }))
    }
}
