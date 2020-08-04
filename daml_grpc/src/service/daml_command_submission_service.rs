use crate::data::DamlCommands;
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::grpc_protobuf::com::daml::ledger::api::v1::command_submission_service_client::CommandSubmissionServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{Commands, SubmitRequest, TraceContext};
use crate::service::common::make_request;
use log::{debug, trace};
use std::convert::TryFrom;
use tonic::transport::Channel;

/// Advance the state of a DAML ledger by submitting commands.
pub struct DamlCommandSubmissionService<'a> {
    channel: Channel,
    ledger_id: &'a str,
    auth_token: Option<&'a str>,
}

impl<'a> DamlCommandSubmissionService<'a> {
    pub fn new(channel: Channel, ledger_id: &'a str, auth_token: Option<&'a str>) -> Self {
        Self {
            channel,
            ledger_id,
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

    /// Override the ledger id to use for this service.
    pub fn with_ledger_id(self, ledger_id: &'a str) -> Self {
        Self {
            ledger_id,
            ..self
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
        self.client().submit(make_request(payload, self.auth_token.as_deref())?).await.map_err(DamlError::from)?;
        Ok(command_id)
    }

    fn client(&self) -> CommandSubmissionServiceClient<Channel> {
        CommandSubmissionServiceClient::new(self.channel.clone())
    }

    // Convert into a GRPC `Commands` and inject the ledger id
    fn create_ledger_commands(&self, commands: DamlCommands) -> DamlResult<Commands> {
        let mut commands = Commands::try_from(commands)?;
        commands.ledger_id = self.ledger_id.to_string();
        Ok(commands)
    }
}
