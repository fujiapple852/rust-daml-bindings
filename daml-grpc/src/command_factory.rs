use crate::data::command::DamlCommand;
use crate::data::{DamlCommands, DamlCommandsDeduplicationPeriod, DamlMinLedgerTime};
use uuid::Uuid;

/// Factory for creating [`DamlCommands`] to submit to a Daml ledger.
#[derive(Debug)]
pub struct DamlCommandFactory {
    workflow_id: String,
    application_id: String,
    act_as: Vec<String>,
    read_as: Vec<String>,
    deduplication_period: Option<DamlCommandsDeduplicationPeriod>,
    min_ledger_time: Option<DamlMinLedgerTime>,
}

impl DamlCommandFactory {
    pub fn new(
        workflow_id: impl Into<String>,
        application_id: impl Into<String>,
        act_as: impl Into<Vec<String>>,
        read_as: impl Into<Vec<String>>,
        deduplication_period: impl Into<Option<DamlCommandsDeduplicationPeriod>>,
        min_ledger_time: impl Into<Option<DamlMinLedgerTime>>,
    ) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            application_id: application_id.into(),
            act_as: act_as.into(),
            read_as: read_as.into(),
            deduplication_period: deduplication_period.into(),
            min_ledger_time: min_ledger_time.into(),
        }
    }

    pub const fn workflow_id(&self) -> &String {
        &self.workflow_id
    }

    pub fn application_id(&self) -> &str {
        &self.application_id
    }

    pub fn act_as(&self) -> &[String] {
        &self.act_as
    }

    pub fn read_as(&self) -> &[String] {
        &self.read_as
    }

    pub const fn deduplication_period(&self) -> &Option<DamlCommandsDeduplicationPeriod> {
        &self.deduplication_period
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
            "",
            "",
            self.act_as.clone(),
            self.read_as.clone(),
            commands.into(),
            self.deduplication_period.clone(),
            self.min_ledger_time.clone(),
        )
    }
}
