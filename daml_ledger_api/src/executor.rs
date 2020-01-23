use crate::data::command::{DamlCommand, DamlCreateCommand, DamlExerciseCommand};
use crate::data::event::{DamlCreatedEvent, DamlTreeEvent};
use crate::data::value::DamlValue;
use crate::data::{DamlError, DamlResult, DamlTransaction, DamlTransactionTree};
use crate::util::Required;
use crate::{DamlCommandFactory, DamlLedgerClient};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::ops::Add;
use time::Duration;

pub enum DamlLedgerTimeMode {
    Static,
    Wallclock,
}

pub struct DamlSimpleExecutorBuilder<'a> {
    ledger_client: &'a DamlLedgerClient,
    acting_party: &'a str,
    time_mode: Option<&'a DamlLedgerTimeMode>,
    workflow_id: Option<&'a str>,
    application_id: Option<&'a str>,
    submission_window_secs: Option<i64>,
}

impl<'a> DamlSimpleExecutorBuilder<'a> {
    pub fn new(ledger_client: &'a DamlLedgerClient, acting_party: &'a str) -> Self {
        Self {
            ledger_client,
            acting_party,
            time_mode: None,
            workflow_id: None,
            application_id: None,
            submission_window_secs: None,
        }
    }

    pub fn time_mode(self, time_mode: &'a DamlLedgerTimeMode) -> Self {
        Self {
            time_mode: Some(time_mode),
            ..self
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

    pub fn submission_window_secs(self, submission_window_secs: i64) -> Self {
        Self {
            submission_window_secs: Some(submission_window_secs),
            ..self
        }
    }

    pub fn build(self) -> DamlSimpleExecutor<'a> {
        DamlSimpleExecutor::new(
            self.ledger_client,
            self.acting_party,
            self.time_mode.unwrap_or(&DamlLedgerTimeMode::Static),
            self.workflow_id.unwrap_or("default-workflow"),
            self.application_id.unwrap_or("default-application"),
            self.submission_window_secs.unwrap_or(30),
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
    ledger_client: &'a DamlLedgerClient,
    command_factory: DamlCommandFactory,
}

impl<'a> DamlSimpleExecutor<'a> {
    pub fn new(
        ledger_client: &'a DamlLedgerClient,
        acting_party: &str,
        time_mode: &DamlLedgerTimeMode,
        workflow_id: &str,
        application_id: &str,
        submission_window_secs: i64,
    ) -> Self {
        let (ledger_effective_time, maximum_record_time) = match time_mode {
            DamlLedgerTimeMode::Static => {
                let ledger_effective_time: DateTime<Utc> =
                    "1970-01-01T00:00:00Z".parse::<DateTime<Utc>>().expect("invalid datetime");
                let maximum_record_time = ledger_effective_time.add(Duration::seconds(submission_window_secs));
                (ledger_effective_time, maximum_record_time)
            },
            _ => panic!("Wallclock time not yet supported"),
        };

        let command_factory = DamlCommandFactory::new(
            workflow_id,
            application_id,
            acting_party,
            ledger_effective_time,
            maximum_record_time,
        );

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
                        DamlTreeEvent::Exercised(exercised_event) => Some(exercised_event.exercise_result),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .req()
    }
}
