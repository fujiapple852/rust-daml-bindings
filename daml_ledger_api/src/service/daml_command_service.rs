use futures::Future;
use grpcio::Channel;
use grpcio::ClientUnaryReceiver;

use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::data::{DamlCommands, DamlTransaction, DamlTransactionTree};
use crate::grpc_protobuf_autogen::command_service::{
    SubmitAndWaitForTransactionIdResponse, SubmitAndWaitForTransactionResponse,
    SubmitAndWaitForTransactionTreeResponse, SubmitAndWaitRequest,
};
use crate::grpc_protobuf_autogen::command_service_grpc::CommandServiceClient;
use crate::grpc_protobuf_autogen::empty::Empty;
use futures::future::{err, ok};
use std::convert::TryInto;

/// Submit commands to a DAML ledger and await the completion.
///
/// The Command Service is able to correlate submitted commands with completion data, identify timeouts, and return
/// contextual information with each tracking result. This supports the implementation of stateless clients.
pub struct DamlCommandService {
    grpc_client: CommandServiceClient,
    ledger_id: String,
}

impl DamlCommandService {
    /// Create a `DamlCommandService` for a given GRPC `channel` and `ledger_id`.
    pub fn new(channel: Channel, ledger_id: String) -> Self {
        Self {
            grpc_client: CommandServiceClient::new(channel),
            ledger_id,
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
    /// future which must be driven to completion before a result can be observed.  See [`submit_and_wait_sync`] for a
    /// version of this method which is submitted _synchronous_ on the _client_ side.
    ///
    /// This method supports server side tracing by providing a [`DamlTraceContext`] via the optional `trace_context`
    /// parameter.
    ///
    /// # Errors
    ///
    /// Propagates failed submissions error and DAML interpretation failures as [`GRPC`] errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use futures::future::Future;
    /// # use chrono::Utc;
    /// # use daml_ledger_api::data::DamlCommands;
    /// # use daml_ledger_api::DamlLedgerClient;
    /// # use daml_ledger_api::data::DamlResult;
    /// # use std::error::Error;
    /// # fn main() -> DamlResult<()> {
    /// let ledger_client = DamlLedgerClient::connect("localhost", 7600)?;
    /// # let commands: DamlCommands = DamlCommands::new("", "", "", "", Utc::now(), Utc::now(), vec![]);
    /// let future_command = ledger_client.command_service().submit_and_wait(commands)?;
    /// match future_command.wait() {
    ///     Ok(command_id) => assert_eq!("1234", command_id),
    ///     Err(e) => panic!(format!("submit_and_wait failed, error was {}", e.description())),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    /// [`DamlCommands`]: crate::data::commands::DamlCommands
    /// [`DamlCommandSubmissionService`]: crate::service::DamlCommandSubmissionService
    /// [`DamlTransactionService`]: crate::service::DamlTransactionService
    /// [`submit_and_wait_sync`]: DamlCommandService::submit_and_wait_sync
    /// [`DamlTraceContext`]: crate::data::trace::DamlTraceContext
    /// [`GRPC`]: crate::data::error::DamlError::GRPC
    pub fn submit_and_wait(
        &self,
        commands: impl Into<DamlCommands>,
    ) -> DamlResult<impl Future<Item = String, Error = DamlError>> {
        self.submit_and_wait_with_trace(commands, None)
    }

    /// Synchronous version of `submit_and_wait` which blocks on the calling thread.
    ///
    /// See [`submit_and_wait`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait`]: DamlCommandService::submit_and_wait
    pub fn submit_and_wait_sync(&self, commands: impl Into<DamlCommands>) -> DamlResult<String> {
        self.submit_and_wait_with_trace(commands, None)?.wait()
    }

    /// Synchronous version of `submit_and_wait_with_trace` which blocks on the calling thread.
    ///
    /// See [`submit_and_wait_with_trace`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_with_trace`]: DamlCommandService::submit_and_wait_with_trace
    pub fn submit_and_wait_with_trace_sync(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<String> {
        self.submit_and_wait_with_trace(commands, trace_context)?.wait()
    }

    /// Execute the `submit_and_wait` method with server side tracing enabled.
    ///
    /// See [`submit_and_wait`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait`]: DamlCommandService::submit_and_wait
    pub fn submit_and_wait_with_trace(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Future<Item = String, Error = DamlError>> {
        let request = self.make_request(commands, trace_context);
        let command_id = request.get_commands().get_command_id().to_owned();
        let async_response: ClientUnaryReceiver<Empty> = self.grpc_client.submit_and_wait_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|_| command_id))
    }

    /// Submits a composite [`DamlCommands`] and returns the resulting transaction id.
    ///
    /// TODO fully document this
    pub fn submit_and_wait_for_transaction_id(
        &self,
        commands: impl Into<DamlCommands>,
    ) -> DamlResult<impl Future<Item = String, Error = DamlError>> {
        self.submit_and_wait_for_transaction_id_with_trace(commands, None)
    }

    /// Synchronous version of `submit_and_wait_for_transaction_id` which blocks on the calling thread.
    ///
    /// See [`submit_and_wait_for_transaction_id`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction_id`]: DamlCommandService::submit_and_wait_for_transaction_id
    pub fn submit_and_wait_for_transaction_id_sync(&self, commands: impl Into<DamlCommands>) -> DamlResult<String> {
        self.submit_and_wait_for_transaction_id_with_trace(commands, None)?.wait()
    }

    /// Synchronous version of `submit_and_wait_for_transaction_id_with_trace` which blocks on the calling thread.
    ///
    /// See [`submit_and_wait_for_transaction_id_with_trace`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction_id_with_trace`]:
    /// DamlCommandService::submit_and_wait_for_transaction_id_with_trace
    pub fn submit_and_wait_for_transaction_id_with_trace_sync(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<String> {
        self.submit_and_wait_for_transaction_id_with_trace(commands, trace_context)?.wait()
    }

    /// Execute the `submit_and_wait_for_transaction_id` method with server side tracing enabled.
    ///
    /// See [`submit_and_wait_for_transaction_id`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction_id`]: DamlCommandService::submit_and_wait_for_transaction_id
    pub fn submit_and_wait_for_transaction_id_with_trace(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Future<Item = String, Error = DamlError>> {
        let request = self.make_request(commands, trace_context);
        let async_response: ClientUnaryReceiver<SubmitAndWaitForTransactionIdResponse> =
            self.grpc_client.submit_and_wait_for_transaction_id_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|r| r.get_transaction_id().to_owned()))
    }

    /// Submits a composite [`DamlCommands`] and returns the resulting [`DamlTransaction`].
    ///
    /// TODO fully document this
    pub fn submit_and_wait_for_transaction(
        &self,
        commands: impl Into<DamlCommands>,
    ) -> DamlResult<impl Future<Item = DamlTransaction, Error = DamlError>> {
        self.submit_and_wait_for_transaction_with_trace(commands, None)
    }

    /// Synchronous version of `submit_and_wait_for_transaction` which blocks on the calling thread.
    ///
    /// See [`submit_and_wait_for_transaction`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction`]: DamlCommandService::submit_and_wait_for_transaction
    pub fn submit_and_wait_for_transaction_sync(
        &self,
        commands: impl Into<DamlCommands>,
    ) -> DamlResult<DamlTransaction> {
        self.submit_and_wait_for_transaction_with_trace(commands, None)?.wait()
    }

    /// Synchronous version of `submit_and_wait_for_transaction` which blocks on the calling thread.
    ///
    /// See [`submit_and_wait_for_transaction`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction`]: DamlCommandService::submit_and_wait_for_transaction
    pub fn submit_and_wait_for_transaction_with_trace_sync(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<DamlTransaction> {
        self.submit_and_wait_for_transaction_with_trace(commands, trace_context)?.wait()
    }

    /// Execute the `submit_and_wait_for_transaction` method with server side tracing enabled.
    ///
    /// See [`submit_and_wait_for_transaction`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction`]: DamlCommandService::submit_and_wait_for_transaction
    pub fn submit_and_wait_for_transaction_with_trace(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Future<Item = DamlTransaction, Error = DamlError>> {
        let request = self.make_request(commands, trace_context);
        let async_response: ClientUnaryReceiver<SubmitAndWaitForTransactionResponse> =
            self.grpc_client.submit_and_wait_for_transaction_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|mut r| r.take_transaction().try_into()).and_then(|transaction| {
            match transaction {
                Ok(tx) => ok(tx),
                Err(e) => err(e),
            }
        }))
    }

    /// Submits a composite [`DamlCommands`] and returns the resulting [`DamlTransactionTree`].
    /// TODO fully document this
    pub fn submit_and_wait_for_transaction_tree(
        &self,
        commands: impl Into<DamlCommands>,
    ) -> DamlResult<impl Future<Item = DamlTransactionTree, Error = DamlError>> {
        self.submit_and_wait_for_transaction_tree_with_trace(commands, None)
    }

    /// Synchronous version of `submit_and_wait_for_transaction_tree` which blocks on the calling thread.
    ///
    /// See [`submit_and_wait_for_transaction_tree`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction_tree`]: DamlCommandService::submit_and_wait_for_transaction_tree
    pub fn submit_and_wait_for_transaction_tree_sync(
        &self,
        commands: impl Into<DamlCommands>,
    ) -> DamlResult<DamlTransactionTree> {
        self.submit_and_wait_for_transaction_tree_with_trace(commands, None)?.wait()
    }

    /// Synchronous version of `submit_and_wait_for_transaction_tree` which blocks on the calling thread.
    ///
    /// See [`submit_and_wait_for_transaction_tree`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction_tree`]: DamlCommandService::submit_and_wait_for_transaction_tree
    pub fn submit_and_wait_for_transaction_tree_with_trace_sync(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<DamlTransactionTree> {
        self.submit_and_wait_for_transaction_tree_with_trace(commands, trace_context)?.wait()
    }

    /// Execute the `submit_and_wait_for_transaction_tree` method with server side tracing enabled.
    ///
    /// See [`submit_and_wait_for_transaction_tree`] for details of the behaviour and example usage.
    ///
    /// [`submit_and_wait_for_transaction_tree`]: DamlCommandService::submit_and_wait_for_transaction_tree
    pub fn submit_and_wait_for_transaction_tree_with_trace(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Future<Item = DamlTransactionTree, Error = DamlError>> {
        let request = self.make_request(commands, trace_context);
        let async_response: ClientUnaryReceiver<SubmitAndWaitForTransactionTreeResponse> =
            self.grpc_client.submit_and_wait_for_transaction_tree_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|mut r| r.take_transaction().try_into()).and_then(|transaction| {
            match transaction {
                Ok(tx) => ok(tx),
                Err(e) => err(e),
            }
        }))
    }

    fn make_request(
        &self,
        commands: impl Into<DamlCommands>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> SubmitAndWaitRequest {
        let mut request = SubmitAndWaitRequest::new();
        let commands = commands.into();
        request.set_commands(commands.into());
        request.mut_commands().set_ledger_id(self.ledger_id.clone());
        if let Some(tc) = trace_context.into() {
            request.set_trace_context(tc.into());
        }
        request
    }
}
