use crate::data::command::DamlCommand;
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::{Command, Commands};
use crate::util;
use chrono::DateTime;
use chrono::Utc;
use std::convert::TryFrom;

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

    pub fn workflow_id(&self) -> &str {
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

impl TryFrom<DamlCommands> for Commands {
    type Error = DamlError;

    fn try_from(daml_commands: DamlCommands) -> DamlResult<Commands> {
        Ok(Commands {
            // To allow each `DamlCommands` to be reusable between ledgers The DAML ledger id is updated immediately
            // prior to sending to the server.
            ledger_id: String::new(),
            workflow_id: daml_commands.workflow_id,
            application_id: daml_commands.application_id,
            command_id: daml_commands.command_id,
            party: daml_commands.party,
            ledger_effective_time: Some(util::to_grpc_timestamp(daml_commands.ledger_effective_time)?),
            maximum_record_time: Some(util::to_grpc_timestamp(daml_commands.maximum_record_time)?),
            commands: daml_commands.commands.into_iter().map(Command::from).collect(),
        })
    }
}
