use crate::data::command::DamlCommand;
use crate::grpc_protobuf_autogen::commands::Commands;
use crate::util;
use chrono::DateTime;
use chrono::Utc;

/// A list of DAML commands.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlCommands {
    pub workflow_id: String,
    pub application_id: String,
    pub command_id: String,
    pub party: String,
    pub ledger_effective_time: DateTime<Utc>,
    pub maximum_record_time: DateTime<Utc>,
    pub commands: Vec<DamlCommand>,
}

impl DamlCommands {
    pub fn new(
        workflow_id: impl Into<String>,
        application_id: impl Into<String>,
        command_id: impl Into<String>,
        party: impl Into<String>,
        ledger_effective_time: impl Into<DateTime<Utc>>,
        maximum_record_time: impl Into<DateTime<Utc>>,
        commands: impl Into<Vec<DamlCommand>>,
    ) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            application_id: application_id.into(),
            command_id: command_id.into(),
            party: party.into(),
            ledger_effective_time: ledger_effective_time.into(),
            maximum_record_time: maximum_record_time.into(),
            commands: commands.into(),
        }
    }

    pub fn workflow_id(&self) -> &String {
        &self.workflow_id
    }

    pub fn application_id(&self) -> &str {
        &self.application_id
    }

    pub fn command_id(&self) -> &str {
        &self.command_id
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

    pub fn commands(&self) -> &[DamlCommand] {
        &self.commands
    }
}

impl From<DamlCommands> for Commands {
    fn from(daml_commands: DamlCommands) -> Self {
        let mut commands = Self::new();
        commands.set_application_id(daml_commands.application_id);
        commands.set_command_id(daml_commands.command_id);
        commands.set_party(daml_commands.party);
        commands.set_ledger_effective_time(util::make_timestamp_secs(daml_commands.ledger_effective_time));
        commands.set_maximum_record_time(util::make_timestamp_secs(daml_commands.maximum_record_time));
        commands.set_workflow_id(daml_commands.workflow_id);
        commands.set_commands(daml_commands.commands.into_iter().map(Into::into).collect());
        commands
    }
}
