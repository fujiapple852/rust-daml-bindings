use crate::data::DamlCommands;
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::grpc_protobuf_autogen::command_submission_service::SubmitRequest;
use crate::grpc_protobuf_autogen::command_submission_service_grpc::CommandSubmissionServiceClient;
use crate::grpc_protobuf_autogen::empty::Empty;
use futures::Future;
use grpcio::Channel;
use grpcio::ClientUnaryReceiver;

/// Advance the state of a DAML ledger by submitting commands.
pub struct DamlCommandSubmissionService {
    grpc_client: CommandSubmissionServiceClient,
    ledger_id: String,
}

impl DamlCommandSubmissionService {
    pub fn new(channel: Channel, ledger_id: String) -> Self {
        Self {
            grpc_client: CommandSubmissionServiceClient::new(channel),
            ledger_id,
        }
    }

    /// TODO fully document this
    pub fn submit_request(&self, commands: DamlCommands) -> DamlResult<impl Future<Item = String, Error = DamlError>> {
        self.submit_request_with_trace(commands, None)
    }

    pub fn submit_request_sync(&self, commands: DamlCommands) -> DamlResult<String> {
        self.submit_request_with_trace(commands, None)?.wait()
    }

    pub fn submit_request_with_trace_sync(
        &self,
        commands: DamlCommands,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<String> {
        self.submit_request_with_trace(commands, trace_context)?.wait()
    }

    pub fn submit_request_with_trace(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Future<Item = String, Error = DamlError>> {
        let mut request = SubmitRequest::new();
        let commands = commands.into();
        let command_id = commands.command_id().to_owned();
        request.set_commands(commands.into());
        request.mut_commands().set_ledger_id(self.ledger_id.clone());
        if let Some(tc) = trace_context.into() {
            request.set_trace_context(tc.into());
        }
        let async_response: ClientUnaryReceiver<Empty> = self.grpc_client.submit_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|_| command_id))
    }
}
