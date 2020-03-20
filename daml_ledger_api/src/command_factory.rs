use crate::data::command::DamlCommand;
use crate::data::DamlCommands;
use chrono::DateTime;
use chrono::Utc;
use std::time::Duration;
use uuid::Uuid;

/// Factory for creating [`DamlCommands`] to submit to a DAML ledger.
// TODO ledger_effective_time / maximum_record_time may be deprecated and replaced with deduplication_time (mutually
// exclusive?)
#[derive(Debug)]
pub struct DamlCommandFactory {
    workflow_id: String,
    application_id: String,
    party: String,
    ledger_effective_time: DateTime<Utc>,
    maximum_record_time: DateTime<Utc>,
    deduplication_time: Option<Duration>,
}

impl DamlCommandFactory {
    pub fn new(
        workflow_id: impl Into<String>,
        application_id: impl Into<String>,
        party: impl Into<String>,
        ledger_effective_time: impl Into<DateTime<Utc>>,
        maximum_record_time: impl Into<DateTime<Utc>>,
        deduplication_time: impl Into<Option<Duration>>,
    ) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            application_id: application_id.into(),
            party: party.into(),
            ledger_effective_time: ledger_effective_time.into(),
            maximum_record_time: maximum_record_time.into(),
            deduplication_time: deduplication_time.into(),
        }
    }

    pub fn workflow_id(&self) -> &String {
        &self.workflow_id
    }

    pub fn application_id(&self) -> &str {
        &self.application_id
    }

    pub fn party(&self) -> &str {
        &self.party
    }

    pub fn ledger_effective_time(&self) -> &DateTime<Utc> {
        &self.ledger_effective_time
    }

    pub fn maximum_record_time(&self) -> &DateTime<Utc> {
        &self.maximum_record_time
    }

    pub fn make_command(&self, command: DamlCommand) -> DamlCommands {
        self.make_commands::<String, _>(vec![command], None)
    }

    pub fn make_command_with_id(&self, command: DamlCommand, command_id: impl Into<String>) -> DamlCommands {
        self.make_commands(vec![command], Some(command_id))
    }

    pub fn deduplication_time(&self) -> &Option<Duration> {
        &self.deduplication_time
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
            self.ledger_effective_time,
            self.maximum_record_time,
            commands.into(),
            self.deduplication_time,
        )
    }
}
