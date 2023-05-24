use async_trait::async_trait;

use crate::data::command::{DamlCommand, DamlCreateCommand, DamlExerciseCommand};
use crate::data::event::{DamlCreatedEvent, DamlTreeEvent};
use crate::data::value::DamlValue;
use crate::data::{
    DamlCommandsDeduplicationPeriod, DamlError, DamlMinLedgerTime, DamlResult, DamlTransaction, DamlTransactionTree,
};
use crate::service::DamlCommandService;
use crate::util::Required;
use crate::{DamlCommandFactory, DamlGrpcClient};

/// Construct a [`DamlSimpleExecutor`].
pub struct DamlSimpleExecutorBuilder<'a> {
    ledger_client: &'a DamlGrpcClient,
    act_as: Option<Vec<String>>,
    read_as: Option<Vec<String>>,
    workflow_id: Option<&'a str>,
    application_id: Option<&'a str>,
    deduplication_period: Option<DamlCommandsDeduplicationPeriod>,
    min_ledger_time: Option<DamlMinLedgerTime>,
    auth_token: Option<&'a str>,
}

impl<'a> DamlSimpleExecutorBuilder<'a> {
    pub const fn new(ledger_client: &'a DamlGrpcClient) -> Self {
        Self {
            ledger_client,
            act_as: None,
            read_as: None,
            workflow_id: None,
            application_id: None,
            deduplication_period: None,
            min_ledger_time: None,
            auth_token: None,
        }
    }

    pub fn workflow_id(self, workflow_id: &'a str) -> Self {
        Self {
            workflow_id: Some(workflow_id),
            ..self
        }
    }

    pub fn act_as(self, act_as: impl Into<String>) -> Self {
        Self {
            act_as: Some(vec![act_as.into()]),
            ..self
        }
    }

    pub fn act_as_all(self, act_as_all: Vec<String>) -> Self {
        Self {
            act_as: Some(act_as_all),
            ..self
        }
    }

    pub fn read_as(self, read_as: impl Into<String>) -> Self {
        Self {
            read_as: Some(vec![read_as.into()]),
            ..self
        }
    }

    pub fn read_as_all(self, read_as_all: Vec<String>) -> Self {
        Self {
            read_as: Some(read_as_all),
            ..self
        }
    }

    pub fn application_id(self, application_id: &'a str) -> Self {
        Self {
            application_id: Some(application_id),
            ..self
        }
    }

    pub fn deduplication_period(self, deduplication_period: DamlCommandsDeduplicationPeriod) -> Self {
        Self {
            deduplication_period: Some(deduplication_period),
            ..self
        }
    }

    pub fn min_ledger_time(self, min_ledger_time: DamlMinLedgerTime) -> Self {
        Self {
            min_ledger_time: Some(min_ledger_time),
            ..self
        }
    }

    /// Override any JWT token enabled in the `DamlGrpcClient`.
    pub fn auth_token(self, auth_token: &'a str) -> Self {
        Self {
            auth_token: Some(auth_token),
            ..self
        }
    }

    pub fn build(self) -> DamlResult<DamlSimpleExecutor<'a>> {
        if self.has_parties() {
            Ok(DamlSimpleExecutor::new(
                self.ledger_client,
                self.act_as.unwrap_or_default(),
                self.read_as.unwrap_or_default(),
                self.workflow_id.unwrap_or("default-workflow"),
                self.application_id.unwrap_or("default-application"),
                self.deduplication_period,
                self.min_ledger_time,
                self.auth_token,
            ))
        } else {
            Err(DamlError::InsufficientParties)
        }
    }

    fn has_parties(&self) -> bool {
        match (self.act_as.as_deref(), self.read_as.as_deref()) {
            (None, None) => false,
            (Some(act_as), None) => !act_as.is_empty(),
            (None, Some(read_as)) => !read_as.is_empty(),
            (Some(act_as), Some(read_as)) => !act_as.is_empty() || !read_as.is_empty(),
        }
    }
}

/// A generic failable executor.
#[async_trait]
pub trait Executor {
    fn execute<T, F: FnOnce(&Self) -> DamlResult<T>>(&self, f: F) -> DamlResult<T> {
        f(self)
    }
}

/// An async failable Daml command executor.
#[async_trait]
pub trait CommandExecutor {
    async fn execute_for_transaction(&self, command: DamlCommand) -> DamlResult<DamlTransaction>;
    async fn execute_for_transaction_tree(&self, command: DamlCommand) -> DamlResult<DamlTransactionTree>;
    async fn execute_create(&self, create_command: DamlCreateCommand) -> DamlResult<DamlCreatedEvent>;
    async fn execute_exercise(&self, exercise_command: DamlExerciseCommand) -> DamlResult<DamlValue>;
}

/// A simple async Daml command executor.
pub struct DamlSimpleExecutor<'a> {
    ledger_client: &'a DamlGrpcClient,
    command_factory: DamlCommandFactory,
    auth_token: Option<&'a str>,
}

impl<'a> DamlSimpleExecutor<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ledger_client: &'a DamlGrpcClient,
        act_as: Vec<String>,
        read_as: Vec<String>,
        workflow_id: &str,
        application_id: &str,
        deduplication_period: Option<DamlCommandsDeduplicationPeriod>,
        min_ledger_time: Option<DamlMinLedgerTime>,
        auth_token: Option<&'a str>,
    ) -> Self {
        let command_factory = DamlCommandFactory::new(
            workflow_id,
            application_id,
            act_as,
            read_as,
            deduplication_period,
            min_ledger_time,
        );
        Self {
            ledger_client,
            command_factory,
            auth_token,
        }
    }

    pub fn act_as(&self) -> &[String] {
        self.command_factory.act_as()
    }

    pub fn read_as(&self) -> &[String] {
        self.command_factory.read_as()
    }

    async fn submit_and_wait_for_transaction(&self, command: DamlCommand) -> DamlResult<DamlTransaction> {
        let commands = self.command_factory.make_command(command);
        Ok(self.client().submit_and_wait_for_transaction(commands).await?.0)
    }

    async fn submit_and_wait_for_transaction_tree(&self, command: DamlCommand) -> DamlResult<DamlTransactionTree> {
        let commands = self.command_factory.make_command(command);
        Ok(self.client().submit_and_wait_for_transaction_tree(commands).await?.0)
    }

    fn client(&self) -> DamlCommandService<'_> {
        match self.auth_token {
            Some(token) => self.ledger_client.command_service().with_token(token),
            None => self.ledger_client.command_service(),
        }
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
        let root_event_id = tx.root_event_ids()[0].clone();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_act_as() -> DamlResult<()> {
        let client = DamlGrpcClient::dummy_for_testing();
        let executor = DamlSimpleExecutorBuilder::new(&client).act_as("Alice").build()?;
        assert_eq!(&["Alice"], executor.act_as());
        assert_eq!(0, executor.read_as().len());
        Ok(())
    }

    #[tokio::test]
    async fn test_read_as() -> DamlResult<()> {
        let client = DamlGrpcClient::dummy_for_testing();
        let executor = DamlSimpleExecutorBuilder::new(&client).read_as("Alice").build()?;
        assert_eq!(&["Alice"], executor.read_as());
        assert_eq!(0, executor.act_as().len());
        Ok(())
    }

    #[tokio::test]
    async fn test_act_as_and_read_as() -> DamlResult<()> {
        let client = DamlGrpcClient::dummy_for_testing();
        let executor = DamlSimpleExecutorBuilder::new(&client).act_as("Alice").read_as("Bob").build()?;
        assert_eq!(&["Alice"], executor.act_as());
        assert_eq!(&["Bob"], executor.read_as());
        Ok(())
    }

    #[tokio::test]
    async fn test_act_as_all() -> DamlResult<()> {
        let client = DamlGrpcClient::dummy_for_testing();
        let executor =
            DamlSimpleExecutorBuilder::new(&client).act_as_all(vec!["Alice".into(), "Bob".into()]).build()?;
        assert_eq!(&["Alice", "Bob"], executor.act_as());
        assert_eq!(0, executor.read_as().len());
        Ok(())
    }

    #[tokio::test]
    async fn test_read_as_all() -> DamlResult<()> {
        let client = DamlGrpcClient::dummy_for_testing();
        let executor =
            DamlSimpleExecutorBuilder::new(&client).read_as_all(vec!["Alice".into(), "Bob".into()]).build()?;
        assert_eq!(&["Alice", "Bob"], executor.read_as());
        assert_eq!(0, executor.act_as().len());
        Ok(())
    }

    #[tokio::test]
    async fn test_act_as_all_and_read_as_all() -> DamlResult<()> {
        let client = DamlGrpcClient::dummy_for_testing();
        let executor = DamlSimpleExecutorBuilder::new(&client)
            .act_as_all(vec!["Alice".into(), "Bob".into()])
            .read_as_all(vec!["John".into(), "Jill".into()])
            .build()?;
        assert_eq!(&["Alice", "Bob"], executor.act_as());
        assert_eq!(&["John", "Jill"], executor.read_as());
        Ok(())
    }

    #[tokio::test]
    async fn test_no_actors_should_fail() -> DamlResult<()> {
        let client = DamlGrpcClient::dummy_for_testing();
        let executor = DamlSimpleExecutorBuilder::new(&client).build();
        match executor {
            Err(DamlError::InsufficientParties) => (),
            _ => panic!("expected DamlError::InsufficientParties"),
        };
        Ok(())
    }
}
