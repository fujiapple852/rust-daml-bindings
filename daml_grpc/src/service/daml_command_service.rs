use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::data::{DamlCommands, DamlTransaction, DamlTransactionTree};
use crate::grpc_protobuf::com::daml::ledger::api::v1::command_service_client::CommandServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{Commands, SubmitAndWaitRequest, TraceContext};
use crate::service::common::make_request;
use crate::util::Required;
use std::convert::TryFrom;
use std::fmt::Debug;
use tonic::transport::Channel;
use tracing::{instrument, trace};

/// Submit commands to a DAML ledger and await the completion.
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
    /// This method supports server side tracing by providing a [`DamlTraceContext`] via the optional `trace_context`
    /// parameter.
    ///
    /// # Errors
    ///
    /// Propagates communication failure errors as [`GRPCTransportError`] and DAML server failures as
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
    /// futures::executor::block_on(async {
    /// let ledger_client = DamlGrpcClientBuilder::uri("http://127.0.0.1").connect().await?;
    /// # let commands: DamlCommands = DamlCommands::new("", "", "", "", vec![], vec![], vec![], None, None);
    /// let future_command = ledger_client.command_service().submit_and_wait(commands).await;
    /// match future_command {
    ///     Ok(command_id) => assert_eq!("1234", command_id),
    ///     Err(e) => panic!("submit_and_wait failed, error was {}", e.to_string()),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// [`DamlCommands`]: crate::data::commands::DamlCommands
    /// [`DamlCommandSubmissionService`]: crate::service::DamlCommandSubmissionService
    /// [`DamlTransactionService`]: crate::service::DamlTransactionService
    /// [`submit_and_wait`]: DamlCommandService::submit_and_wait
    /// [`DamlTraceContext`]: crate::data::trace::DamlTraceContext
    /// [`GRPCTransportError`]: crate::data::error::DamlError::GRPCTransportError
    /// [`GRPCStatusError`]: crate::data::error::DamlError::GRPCStatusError
    #[instrument(skip(self, commands))]
    pub async fn submit_and_wait(&self, commands: impl Into<DamlCommands> + Debug) -> DamlResult<String> {
        self.submit_and_wait_with_trace(commands, None).await
    }

    /// Execute the `submit_and_wait` method with server side tracing enabled.
    ///
    /// See [`submit_and_wait`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait`]: DamlCommandService::submit_and_wait
    #[instrument(skip(self))]
    pub async fn submit_and_wait_with_trace(
        &self,
        commands: impl Into<DamlCommands> + Debug,
        trace_context: impl Into<Option<DamlTraceContext>> + Debug,
    ) -> DamlResult<String> {
        let commands = commands.into();
        let command_id = commands.command_id().to_owned();
        let payload = self.make_payload(commands, trace_context)?;
        trace!(payload = ?payload, token = ?self.auth_token);
        self.client().submit_and_wait(make_request(payload, self.auth_token)?).await?;
        Ok(command_id)
    }

    /// Submits a composite [`DamlCommands`] and returns the resulting transaction id.
    ///
    /// DOCME fully document this
    #[instrument(skip(self, commands))]
    pub async fn submit_and_wait_for_transaction_id(
        &self,
        commands: impl Into<DamlCommands> + Debug,
    ) -> DamlResult<String> {
        self.submit_and_wait_for_transaction_id_with_trace(commands, None).await
    }

    /// Execute the `submit_and_wait_for_transaction_id` method with server side tracing enabled.
    ///
    /// See [`submit_and_wait_for_transaction_id`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction_id`]: DamlCommandService::submit_and_wait_for_transaction_id
    #[instrument(skip(self))]
    pub async fn submit_and_wait_for_transaction_id_with_trace(
        &self,
        commands: impl Into<DamlCommands> + Debug,
        trace_context: impl Into<Option<DamlTraceContext>> + Debug,
    ) -> DamlResult<String> {
        let payload = self.make_payload(commands, trace_context)?;
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self
            .client()
            .submit_and_wait_for_transaction_id(make_request(payload, self.auth_token)?)
            .await?
            .into_inner();
        trace!(?response);
        Ok(response.transaction_id)
    }

    /// Submits a composite [`DamlCommands`] and returns the resulting [`DamlTransaction`].
    ///
    /// DOCME fully document this
    #[instrument(skip(self, commands))]
    pub async fn submit_and_wait_for_transaction(
        &self,
        commands: impl Into<DamlCommands> + Debug,
    ) -> DamlResult<DamlTransaction> {
        self.submit_and_wait_for_transaction_with_trace(commands, None).await
    }

    /// Execute the `submit_and_wait_for_transaction` method with server side tracing enabled.
    ///
    /// See [`submit_and_wait_for_transaction`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction`]: DamlCommandService::submit_and_wait_for_transaction
    #[instrument(skip(self))]
    pub async fn submit_and_wait_for_transaction_with_trace(
        &self,
        commands: impl Into<DamlCommands> + Debug,
        trace_context: impl Into<Option<DamlTraceContext>> + Debug,
    ) -> DamlResult<DamlTransaction> {
        let payload = self.make_payload(commands, trace_context)?;
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().submit_and_wait_for_transaction(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        response.transaction.req().and_then(DamlTransaction::try_from)
    }

    /// Submits a composite [`DamlCommands`] and returns the resulting [`DamlTransactionTree`].
    /// DOCME fully document this
    #[instrument(skip(self, commands))]
    pub async fn submit_and_wait_for_transaction_tree(
        &self,
        commands: impl Into<DamlCommands> + Debug,
    ) -> DamlResult<DamlTransactionTree> {
        self.submit_and_wait_for_transaction_tree_with_trace(commands, None).await
    }

    /// Execute the `submit_and_wait_for_transaction_tree` method with server side tracing enabled.
    ///
    /// See [`submit_and_wait_for_transaction_tree`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction_tree`]: DamlCommandService::submit_and_wait_for_transaction_tree
    #[instrument(skip(self))]
    pub async fn submit_and_wait_for_transaction_tree_with_trace(
        &self,
        commands: impl Into<DamlCommands> + Debug,
        trace_context: impl Into<Option<DamlTraceContext>> + Debug,
    ) -> DamlResult<DamlTransactionTree> {
        let payload = self.make_payload(commands, trace_context)?;
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self
            .client()
            .submit_and_wait_for_transaction_tree(make_request(payload, self.auth_token)?)
            .await?
            .into_inner();
        trace!(?response);
        response.transaction.req().and_then(DamlTransactionTree::try_from)
    }

    fn client(&self) -> CommandServiceClient<Channel> {
        CommandServiceClient::new(self.channel.clone())
    }

    fn make_payload(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<SubmitAndWaitRequest> {
        let mut commands = Commands::try_from(commands.into())?;
        commands.ledger_id = self.ledger_id.to_string();
        Ok(SubmitAndWaitRequest {
            commands: Some(commands),
            trace_context: trace_context.into().map(TraceContext::from),
        })
    }
}
