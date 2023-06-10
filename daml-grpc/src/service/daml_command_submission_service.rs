use std::convert::TryFrom;
use std::fmt::Debug;

use crate::data::DamlCommands;
use crate::data::DamlResult;
use crate::grpc_protobuf::com::daml::ledger::api::v1::command_submission_service_client::CommandSubmissionServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{Commands, SubmitRequest};
use crate::make_grpc_error_type;
use crate::service::common::{make_request2, GrpcApiError, GrpcCode};
use tonic::transport::Channel;
use tracing::{instrument, trace};

/// Advance the state of a Daml ledger by submitting commands.
#[derive(Debug)]
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
    #[instrument(skip(self))]
    pub async fn submit_request(&self, commands: impl Into<DamlCommands> + Debug) -> DamlSubmitRequestResult<String> {
        let commands = commands.into();
        let command_id = commands.command_id().to_owned();
        let payload = SubmitRequest {
            commands: Some(self.create_ledger_commands(commands)?),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        self.client().submit(make_request2(payload, self.auth_token)?).await?;
        trace!(?command_id);
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

make_grpc_error_type!(DamlSubmitRequestErrorCodes, DamlSubmitRequestError, DamlSubmitRequestResult,
      NotFound => "the request does not include a valid ledger id"
    | InvalidArgument => "the payload is malformed or is missing required fields"
    | Unavailable => "the participant is not yet ready to submit commands or if the service has been shut down"
    | ResourceExhausted => "the participant or the ledger is overloaded. Clients should back off exponentially and retry."
);
