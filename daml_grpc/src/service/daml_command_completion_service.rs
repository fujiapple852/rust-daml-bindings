use std::convert::TryFrom;
use std::fmt::Debug;

use futures::Stream;
use futures::StreamExt;
use tonic::transport::Channel;
use tracing::{instrument, trace};

use crate::data::completion::DamlCompletionResponse;
use crate::data::offset::DamlLedgerOffset;
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::grpc_protobuf::com::daml::ledger::api::v1::command_completion_service_client::CommandCompletionServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{CompletionEndRequest, CompletionStreamRequest, LedgerOffset};
use crate::service::common::make_request;
use crate::util::Required;

/// Observe the status of command submissions on a DAML ledger.
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
    ) -> DamlResult<impl Stream<Item = DamlResult<DamlCompletionResponse>>> {
        let payload = CompletionStreamRequest {
            ledger_id: self.ledger_id.to_string(),
            application_id: application_id.into(),
            offset: Some(LedgerOffset::from(offset.into())),
            parties: parties.into(),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let completion_stream =
            self.client().completion_stream(make_request(payload, self.auth_token)?).await?.into_inner();
        Ok(completion_stream.inspect(|response| trace!(?response)).map(|item| match item {
            Ok(completion) => DamlCompletionResponse::try_from(completion),
            Err(e) => Err(DamlError::from(e)),
        }))
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_completion_end(&self) -> DamlResult<DamlLedgerOffset> {
        let payload = CompletionEndRequest {
            ledger_id: self.ledger_id.to_string(),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self
            .client()
            .completion_end(make_request(payload, self.auth_token)?)
            .await
            .map_err(DamlError::from)?
            .into_inner();
        trace!(?response);
        response.offset.req().map(DamlLedgerOffset::try_from)?
    }

    fn client(&self) -> CommandCompletionServiceClient<Channel> {
        CommandCompletionServiceClient::new(self.channel.clone())
    }
}
