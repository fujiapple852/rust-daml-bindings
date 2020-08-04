use crate::data::command::{DamlCommand, DamlCreateCommand, DamlExerciseCommand};
use crate::data::event::{DamlCreatedEvent, DamlTreeEvent};
use crate::data::value::DamlValue;
use crate::data::{DamlError, DamlMinLedgerTime, DamlResult, DamlTransaction, DamlTransactionTree};
use crate::util::Required;
use crate::{DamlCommandFactory, DamlGrpcClient};
use async_trait::async_trait;
use std::time::Duration;

pub struct DamlSimpleExecutorBuilder<'a> {
    ledger_client: &'a DamlGrpcClient,
    acting_party: &'a str,
    workflow_id: Option<&'a str>,
    application_id: Option<&'a str>,
    deduplication_time: Option<Duration>,
    min_ledger_time: Option<DamlMinLedgerTime>,
}

impl<'a> DamlSimpleExecutorBuilder<'a> {
    pub const fn new(ledger_client: &'a DamlGrpcClient, acting_party: &'a str) -> Self {
        Self {
            ledger_client,
            acting_party,
            workflow_id: None,
            application_id: None,
            deduplication_time: None,
            min_ledger_time: None,
        }
    }

    pub fn workflow_id(self, workflow_id: &'a str) -> Self {
        Self {
            workflow_id: Some(workflow_id),
            ..self
        }
    }

    pub fn application_id(self, application_id: &'a str) -> Self {
        Self {
            application_id: Some(application_id),
            ..self
        }
    }

    pub fn deduplication_time(self, deduplication_time: Duration) -> Self {
        Self {
            deduplication_time: Some(deduplication_time),
            ..self
        }
    }

    pub fn min_ledger_time(self, min_ledger_time: DamlMinLedgerTime) -> Self {
        Self {
            min_ledger_time: Some(min_ledger_time),
            ..self
        }
    }

    pub fn build(self) -> DamlSimpleExecutor<'a> {
        DamlSimpleExecutor::new(
            self.ledger_client,
            self.acting_party,
            self.workflow_id.unwrap_or("default-workflow"),
            self.application_id.unwrap_or("default-application"),
            self.deduplication_time,
            self.min_ledger_time,
        )
    }
}

#[async_trait]
pub trait Executor {
    fn execute<T, F: FnOnce(&Self) -> DamlResult<T>>(&self, f: F) -> DamlResult<T> {
        f(self)
    }
}

#[async_trait]
pub trait CommandExecutor {
    async fn execute_for_transaction(&self, command: DamlCommand) -> DamlResult<DamlTransaction>;
    async fn execute_for_transaction_tree(&self, command: DamlCommand) -> DamlResult<DamlTransactionTree>;
    async fn execute_create(&self, create_command: DamlCreateCommand) -> DamlResult<DamlCreatedEvent>;
    async fn execute_exercise(&self, exercise_command: DamlExerciseCommand) -> DamlResult<DamlValue>;
}

pub struct DamlSimpleExecutor<'a> {
    ledger_client: &'a DamlGrpcClient,
    command_factory: DamlCommandFactory,
}

impl<'a> DamlSimpleExecutor<'a> {
    pub fn new(
        ledger_client: &'a DamlGrpcClient,
        acting_party: &str,
        workflow_id: &str,
        application_id: &str,
        deduplication_time: Option<Duration>,
        min_ledger_time: Option<DamlMinLedgerTime>,
    ) -> Self {
        let command_factory =
            DamlCommandFactory::new(workflow_id, application_id, acting_party, deduplication_time, min_ledger_time);
        Self {
            ledger_client,
            command_factory,
        }
    }

    async fn submit_and_wait_for_transaction(&self, command: DamlCommand) -> DamlResult<DamlTransaction> {
        let commands = self.command_factory.make_command(command);
        self.ledger_client.command_service().submit_and_wait_for_transaction(commands).await
    }

    async fn submit_and_wait_for_transaction_tree(&self, command: DamlCommand) -> DamlResult<DamlTransactionTree> {
        let commands = self.command_factory.make_command(command);
        self.ledger_client.command_service().submit_and_wait_for_transaction_tree(commands).await
    }
}

impl Executor for DamlSimpleExecutor<'_> {}

#[async_trait]
#[allow(clippy::needless_lifetimes)]
impl CommandExecutor for DamlSimpleExecutor<'_> {
    async fn execute_for_transaction(&self, command: DamlCommand) -> DamlResult<DamlTransaction> {
        self.submit_and_wait_for_transaction(command).await
    }

    async fn execute_for_transaction_tree(&self, command: DamlCommand) -> DamlResult<DamlTransactionTree> {
        self.submit_and_wait_for_transaction_tree(command).await
    }

    async fn execute_create(&self, create_command: DamlCreateCommand) -> Result<DamlCreatedEvent, DamlError> {
        self.submit_and_wait_for_transaction(DamlCommand::Create(create_command))
            .await?
            .take_events()
            .swap_remove(0)
            .try_created()
    }

    // TODO only takes the first root event
    // TODO abstract away the "find the root" logic
    async fn execute_exercise(&self, exercise_command: DamlExerciseCommand) -> Result<DamlValue, DamlError> {
        let tx = self.submit_and_wait_for_transaction_tree(DamlCommand::Exercise(exercise_command)).await?;
        let root_event_id = tx.root_event_ids()[0].to_owned();
        tx.take_events_by_id()
            .into_iter()
            .find_map(|(id, e)| {
                if id == root_event_id {
                    match e {
                        DamlTreeEvent::Exercised(exercised_event) => Some(exercised_event.take_exercise_result()),
                        DamlTreeEvent::Created(_) => None,
                    }
                } else {
                    None
                }
            })
            .req()
    }
}
