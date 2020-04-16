use crate::data::DamlCommands;
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::grpc_protobuf::com::daml::ledger::api::v1::command_submission_service_client::CommandSubmissionServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{Commands, SubmitRequest, TraceContext};
use crate::ledger_client::DamlTokenRefresh;
use crate::service::common::make_request;
use log::{debug, trace};
use std::convert::TryFrom;
use tonic::transport::Channel;

/// Advance the state of a DAML ledger by submitting commands.
pub struct DamlCommandSubmissionService {
    channel: Channel,
    ledger_id: String,
    auth_token: Option<String>,
}

impl DamlCommandSubmissionService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
            auth_token,
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
        debug!("submit_request");
        let commands = commands.into();
        let command_id = commands.command_id().to_owned();
        let payload = SubmitRequest {
            commands: Some(self.create_ledger_commands(commands)?),
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!("submit_request payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        self.client().submit(make_request(payload, &self.auth_token)?).await.map_err(DamlError::from)?;
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

impl DamlTokenRefresh for DamlCommandSubmissionService {
    fn refresh_token(&mut self, auth_token: Option<String>) {
        self.auth_token = auth_token
    }
}
