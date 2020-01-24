use crate::data::DamlCommands;
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::command_submission_service_client::CommandSubmissionServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::{Commands, SubmitRequest, TraceContext};
use std::convert::TryFrom;
use tonic::transport::Channel;
use tonic::Request;

/// Advance the state of a DAML ledger by submitting commands.
pub struct DamlCommandSubmissionService {
    channel: Channel,
    ledger_id: String,
}

impl DamlCommandSubmissionService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
        }
    }

    /// DOCME fully document this
    pub async fn submit_request(&self, commands: DamlCommands) -> DamlResult<String> {
        self.submit_request_with_trace(commands, None).await
    }

    pub async fn submit_request_with_trace(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<String> {
        let commands = commands.into();
        let command_id = commands.command_id().to_owned();
        let request = Request::new(SubmitRequest {
            commands: Some(self.create_ledger_commands(commands)?),
            trace_context: trace_context.into().map(TraceContext::from),
        });
        self.client().submit(request).await.map_err(DamlError::from)?;
        Ok(command_id)
    }

    fn client(&self) -> CommandSubmissionServiceClient<Channel> {
        CommandSubmissionServiceClient::new(self.channel.clone())
    }

    // Convert into a GRPC `Commands` and inject the ledger id
    fn create_ledger_commands(&self, commands: DamlCommands) -> DamlResult<Commands> {
        let mut commands = Commands::try_from(commands)?;
        commands.ledger_id = self.ledger_id.clone();
        Ok(commands)
    }
}
