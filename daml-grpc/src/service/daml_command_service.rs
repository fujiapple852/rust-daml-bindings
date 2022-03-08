use std::convert::TryFrom;
use std::fmt::Debug;

use tonic::transport::Channel;
use tracing::{instrument, trace};

use crate::data::DamlResult;
use crate::data::{DamlCommands, DamlTransaction, DamlTransactionTree};
use crate::grpc_protobuf::com::daml::ledger::api::v1::command_service_client::CommandServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{Commands, SubmitAndWaitRequest};
use crate::service::common::make_request;
use crate::util::Required;

/// Submit commands to a Daml ledger and await the completion.
///
/// The Command Service is able to correlate submitted commands with completion data, identify timeouts, and return
/// contextual information with each tracking result. This supports the implementation of stateless clients.
#[derive(Debug)]
pub struct DamlCommandService<'a> {
    channel: Channel,
    ledger_id: &'a str,
    auth_token: Option<&'a str>,
}

impl<'a> DamlCommandService<'a> {
    /// Create a `DamlCommandService` for a given GRPC `channel` and `ledger_id`.
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

    /// Submits a composite [`DamlCommands`] and await the completion.
    ///
    /// This method executes `commands` _synchronously_ on the ledger server (unlike the
    /// [`DamlCommandSubmissionService`] which is executed _asynchronously_ on the ledger server).  This service only
    /// waits for the completion of the execution of the command, not the propagation of any resulting events which
    /// must be consumed via the [`DamlTransactionService`].
    ///
    /// Note that this method is executed _asynchronously_ on the _client_ side and so will immediately return a
    /// future which must be driven to completion before a result can be observed.
    ///
    /// # Errors
    ///
    /// Propagates communication failure errors as [`GrpcTransportError`] and Daml server failures as
    /// [`GRPCStatusError`] errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use futures::future::Future;
    /// # use chrono::Utc;
    /// # use daml_grpc::data::DamlCommands;
    /// # use daml_grpc::DamlGrpcClientBuilder;
    /// # use daml_grpc::data::DamlResult;
    /// # use std::error::Error;
    /// # fn main() -> DamlResult<()> {
    /// # futures::executor::block_on(async {
    /// let ledger_client = DamlGrpcClientBuilder::uri("http://127.0.0.1").connect().await?;
    /// # let commands: DamlCommands = DamlCommands::new("", "", "", "", "", vec![], vec![], vec![], None, None);
    /// let future_command = ledger_client.command_service().submit_and_wait(commands).await;
    /// match future_command {
    ///     Ok(command_id) => assert_eq!("1234", command_id),
    ///     Err(e) => panic!("submit_and_wait failed, error was {}", e.to_string()),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// [`DamlCommands`]: crate::data::DamlCommands
    /// [`DamlCommandSubmissionService`]: crate::service::DamlCommandSubmissionService
    /// [`DamlTransactionService`]: crate::service::DamlTransactionService
    /// [`submit_and_wait`]: DamlCommandService::submit_and_wait
    /// [`GrpcTransportError`]: crate::data::DamlError::GrpcTransportError
    /// [`GrpcStatusError`]: crate::data::DamlError::GrpcStatusError
    #[instrument(skip(self))]
    pub async fn submit_and_wait(&self, commands: impl Into<DamlCommands> + Debug) -> DamlResult<String> {
        let commands = commands.into();
        let command_id = commands.command_id().to_owned();
        let payload = self.make_payload(commands)?;
        trace!(payload = ?payload, token = ?self.auth_token);
        self.client().submit_and_wait(make_request(payload, self.auth_token)?).await?;
        Ok(command_id)
    }

    /// Submits a composite [`DamlCommands`] and returns the resulting transaction id.
    ///
    /// DOCME fully document this
    /// TODO ugly API returning a tuple as `completion_offset` was recently added, refactor
    #[instrument(skip(self))]
    pub async fn submit_and_wait_for_transaction_id(
        &self,
        commands: impl Into<DamlCommands> + Debug,
    ) -> DamlResult<(String, String)> {
        let payload = self.make_payload(commands)?;
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self
            .client()
            .submit_and_wait_for_transaction_id(make_request(payload, self.auth_token)?)
            .await?
            .into_inner();
        trace!(?response);
        Ok((response.transaction_id, response.completion_offset))
    }

    /// Submits a composite [`DamlCommands`] and returns the resulting [`DamlTransaction`].
    ///
    /// DOCME fully document this
    /// TODO ugly API returning a tuple as `completion_offset` was recently added, refactor
    #[instrument(skip(self))]
    pub async fn submit_and_wait_for_transaction(
        &self,
        commands: impl Into<DamlCommands> + Debug,
    ) -> DamlResult<(DamlTransaction, String)> {
        let payload = self.make_payload(commands)?;
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().submit_and_wait_for_transaction(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        response.transaction.req().and_then(DamlTransaction::try_from).map(|tree| (tree, response.completion_offset))
    }

    /// Submits a composite [`DamlCommands`] and returns the resulting [`DamlTransactionTree`].
    /// DOCME fully document this
    /// TODO ugly API returning a tuple as `completion_offset` was recently added, refactor
    #[instrument(skip(self))]
    pub async fn submit_and_wait_for_transaction_tree(
        &self,
        commands: impl Into<DamlCommands> + Debug,
    ) -> DamlResult<(DamlTransactionTree, String)> {
        let payload = self.make_payload(commands)?;
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self
            .client()
            .submit_and_wait_for_transaction_tree(make_request(payload, self.auth_token)?)
            .await?
            .into_inner();
        trace!(?response);
        response
            .transaction
            .req()
            .and_then(DamlTransactionTree::try_from)
            .map(|tree| (tree, response.completion_offset))
    }

    fn client(&self) -> CommandServiceClient<Channel> {
        CommandServiceClient::new(self.channel.clone())
    }

    fn make_payload(&self, commands: impl Into<DamlCommands>) -> DamlResult<SubmitAndWaitRequest> {
        let mut commands = Commands::try_from(commands.into())?;
        commands.ledger_id = self.ledger_id.to_string();
        Ok(SubmitAndWaitRequest {
            commands: Some(commands),
        })
    }
}
