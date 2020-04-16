use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::data::{DamlCommands, DamlTransaction, DamlTransactionTree};

use crate::grpc_protobuf::com::daml::ledger::api::v1::command_service_client::CommandServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{Commands, SubmitAndWaitRequest, TraceContext};
use crate::ledger_client::DamlTokenRefresh;
use crate::service::common::make_request;
use crate::util::Required;
use log::{debug, trace};
use std::convert::TryFrom;
use tonic::transport::Channel;

/// Submit commands to a DAML ledger and await the completion.
///
/// The Command Service is able to correlate submitted commands with completion data, identify timeouts, and return
/// contextual information with each tracking result. This supports the implementation of stateless clients.
pub struct DamlCommandService {
    channel: Channel,
    ledger_id: String,
    auth_token: Option<String>,
}

impl DamlCommandService {
    /// Create a `DamlCommandService` for a given GRPC `channel` and `ledger_id`.
    pub fn new(channel: Channel, ledger_id: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
            auth_token,
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
    /// # use daml_api::data::DamlCommands;
    /// # use daml_api::DamlLedgerClientBuilder;
    /// # use daml_api::data::DamlResult;
    /// # use std::error::Error;
    /// # fn main() -> DamlResult<()> {
    /// futures::executor::block_on(async {
    /// let ledger_client = DamlLedgerClientBuilder::uri("http://127.0.0.1").connect().await?;
    /// # let commands: DamlCommands = DamlCommands::new("", "", "", "", vec![], None, None);
    /// let future_command = ledger_client.command_service().submit_and_wait(commands).await;
    /// match future_command {
    ///     Ok(command_id) => assert_eq!("1234", command_id),
    ///     Err(e) => panic!(format!("submit_and_wait failed, error was {}", e.to_string())),
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
    pub async fn submit_and_wait(&self, commands: impl Into<DamlCommands>) -> DamlResult<String> {
        self.submit_and_wait_with_trace(commands, None).await
    }

    /// Execute the `submit_and_wait` method with server side tracing enabled.
    ///
    /// See [`submit_and_wait`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait`]: DamlCommandService::submit_and_wait
    pub async fn submit_and_wait_with_trace(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<String> {
        debug!("submit_and_wait");
        let commands = commands.into();
        let command_id = commands.command_id().to_owned();
        let payload = self.make_payload(commands, trace_context)?;
        trace!("submit_and_wait payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        self.client().submit_and_wait(make_request(payload, &self.auth_token)?).await?;
        Ok(command_id)
    }

    /// Submits a composite [`DamlCommands`] and returns the resulting transaction id.
    ///
    /// DOCME fully document this
    pub async fn submit_and_wait_for_transaction_id(&self, commands: impl Into<DamlCommands>) -> DamlResult<String> {
        self.submit_and_wait_for_transaction_id_with_trace(commands, None).await
    }

    /// Execute the `submit_and_wait_for_transaction_id` method with server side tracing enabled.
    ///
    /// See [`submit_and_wait_for_transaction_id`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction_id`]: DamlCommandService::submit_and_wait_for_transaction_id
    pub async fn submit_and_wait_for_transaction_id_with_trace(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<String> {
        debug!("submit_and_wait_for_transaction_id");
        let payload = self.make_payload(commands, trace_context)?;
        trace!("submit_and_wait_for_transaction_id payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        Ok(self
            .client()
            .submit_and_wait_for_transaction_id(make_request(payload, &self.auth_token)?)
            .await?
            .into_inner()
            .transaction_id)
    }

    /// Submits a composite [`DamlCommands`] and returns the resulting [`DamlTransaction`].
    ///
    /// DOCME fully document this
    pub async fn submit_and_wait_for_transaction(
        &self,
        commands: impl Into<DamlCommands>,
    ) -> DamlResult<DamlTransaction> {
        self.submit_and_wait_for_transaction_with_trace(commands, None).await
    }

    /// Execute the `submit_and_wait_for_transaction` method with server side tracing enabled.
    ///
    /// See [`submit_and_wait_for_transaction`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction`]: DamlCommandService::submit_and_wait_for_transaction
    pub async fn submit_and_wait_for_transaction_with_trace(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<DamlTransaction> {
        debug!("submit_and_wait_for_transaction");
        let payload = self.make_payload(commands, trace_context)?;
        trace!("submit_and_wait_for_transaction payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        self.client()
            .submit_and_wait_for_transaction(make_request(payload, &self.auth_token)?)
            .await?
            .into_inner()
            .transaction
            .req()
            .and_then(DamlTransaction::try_from)
    }

    /// Submits a composite [`DamlCommands`] and returns the resulting [`DamlTransactionTree`].
    /// DOCME fully document this
    pub async fn submit_and_wait_for_transaction_tree(
        &self,
        commands: impl Into<DamlCommands>,
    ) -> DamlResult<DamlTransactionTree> {
        self.submit_and_wait_for_transaction_tree_with_trace(commands, None).await
    }

    /// Execute the `submit_and_wait_for_transaction_tree` method with server side tracing enabled.
    ///
    /// See [`submit_and_wait_for_transaction_tree`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction_tree`]: DamlCommandService::submit_and_wait_for_transaction_tree
    pub async fn submit_and_wait_for_transaction_tree_with_trace(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<DamlTransactionTree> {
        debug!("submit_and_wait_for_transaction_tree");
        let payload = self.make_payload(commands, trace_context)?;
        trace!("submit_and_wait_for_transaction_tree payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        self.client()
            .submit_and_wait_for_transaction_tree(make_request(payload, &self.auth_token)?)
            .await?
            .into_inner()
            .transaction
            .req()
            .and_then(DamlTransactionTree::try_from)
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
        commands.ledger_id = self.ledger_id.clone();
        Ok(SubmitAndWaitRequest {
            commands: Some(commands),
            trace_context: trace_context.into().map(TraceContext::from),
        })
    }
}

impl DamlTokenRefresh for DamlCommandService {
    fn refresh_token(&mut self, auth_token: Option<String>) {
        self.auth_token = auth_token
    }
}
