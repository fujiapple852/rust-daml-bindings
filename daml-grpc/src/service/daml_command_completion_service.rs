use std::convert::TryFrom;
use std::fmt::Debug;

use futures::Stream;
use futures::StreamExt;
use tonic::transport::Channel;
use tracing::{instrument, trace};

use crate::data::completion::DamlCompletionResponse;
use crate::data::offset::DamlLedgerOffset;
use crate::grpc_protobuf::com::daml::ledger::api::v1::command_completion_service_client::CommandCompletionServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{CompletionEndRequest, CompletionStreamRequest, LedgerOffset};
use crate::make_grpc_error_type;
use crate::service::common::{make_request2, trace_item, GrpcApiError, GrpcCode};
use crate::util::Required;

/// Observe the status of command submissions on a Daml ledger.
#[derive(Debug)]
pub struct DamlCommandCompletionService<'a> {
    channel: Channel,
    ledger_id: &'a str,
    auth_token: Option<&'a str>,
}

impl<'a> DamlCommandCompletionService<'a> {
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
    pub async fn get_completion_stream(
        &self,
        application_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
        offset: impl Into<DamlLedgerOffset> + Debug,
    ) -> DamlGetCompletionStreamResult<impl Stream<Item = DamlGetCompletionStreamResult<DamlCompletionResponse>>> {
        let payload = CompletionStreamRequest {
            ledger_id: self.ledger_id.to_string(),
            application_id: application_id.into(),
            offset: Some(LedgerOffset::from(offset.into())),
            parties: parties.into(),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let completion_stream =
            self.client().completion_stream(make_request2(payload, self.auth_token)?).await?.into_inner();
        Ok(completion_stream.inspect(trace_item).map(|item| match item {
            Ok(completion) => Ok(DamlCompletionResponse::try_from(completion)?),
            Err(e) => Err(DamlGetCompletionStreamError::from(e)),
        }))
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_completion_end(&self) -> DamlCompletionEndResult<DamlLedgerOffset> {
        let payload = CompletionEndRequest {
            ledger_id: self.ledger_id.to_string(),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self.client().completion_end(make_request2(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        Ok(DamlLedgerOffset::try_from(response.offset.req()?)?)
    }

    fn client(&self) -> CommandCompletionServiceClient<Channel> {
        CommandCompletionServiceClient::new(self.channel.clone())
    }
}

make_grpc_error_type!(DamlGetCompletionStreamErrorCodes, DamlGetCompletionStreamError, DamlGetCompletionStreamResult,
      NotFound => "the request does not include a valid ledger id or the ledger has been pruned before begin"
    | InvalidArgument => "the payload is malformed or is missing required fields"
    | OutOfRange => "the absolute offset is not before the end of the ledger"
);

make_grpc_error_type!(DamlCompletionEndErrorCodes, DamlCompletionEndError, DamlCompletionEndResult,
      NotFound => "the request does not include a valid ledger id"
);
