use crate::data::command::DamlCommand;
use crate::data::{DamlCommands, DamlMinLedgerTime};
use std::time::Duration;
use uuid::Uuid;

/// Factory for creating [`DamlCommands`] to submit to a DAML ledger.
#[derive(Debug)]
pub struct DamlCommandFactory {
    workflow_id: String,
    application_id: String,
    party: String,
    deduplication_time: Option<Duration>,
    min_ledger_time: Option<DamlMinLedgerTime>,
}

impl DamlCommandFactory {
    pub fn new(
        workflow_id: impl Into<String>,
        application_id: impl Into<String>,
        party: impl Into<String>,
        deduplication_time: impl Into<Option<Duration>>,
        min_ledger_time: impl Into<Option<DamlMinLedgerTime>>,
    ) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            application_id: application_id.into(),
            party: party.into(),
            deduplication_time: deduplication_time.into(),
            min_ledger_time: min_ledger_time.into(),
        }
    }

    pub const fn workflow_id(&self) -> &String {
        &self.workflow_id
    }

    pub fn application_id(&self) -> &str {
        &self.application_id
    }

    pub fn party(&self) -> &str {
        &self.party
    }

    pub const fn deduplication_time(&self) -> &Option<Duration> {
        &self.deduplication_time
    }

    pub const fn min_ledger_time(&self) -> &Option<DamlMinLedgerTime> {
        &self.min_ledger_time
    }

    pub fn make_command(&self, command: DamlCommand) -> DamlCommands {
        self.make_commands::<String, _>(vec![command], None)
    }

    pub fn make_command_with_id(&self, command: DamlCommand, command_id: impl Into<String>) -> DamlCommands {
        self.make_commands(vec![command], Some(command_id))
    }

    pub fn make_commands<S, V>(&self, commands: V, command_id: Option<S>) -> DamlCommands
    where
        S: Into<String>,
        V: Into<Vec<DamlCommand>>,
    {
        DamlCommands::new(
            self.workflow_id.clone(),
            self.application_id.clone(),
            command_id.map_or_else(|| format!("{}", Uuid::new_v4()), Into::into),
            self.party.clone(),
            commands.into(),
            self.deduplication_time,
            self.min_ledger_time.clone(),
        )
    }
}
