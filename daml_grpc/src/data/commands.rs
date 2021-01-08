use crate::data::command::DamlCommand;
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::daml::ledger::api::v1::{Command, Commands};
use crate::util;
use chrono::DateTime;
use chrono::Utc;
use std::convert::TryFrom;
use std::time::Duration;

/// A list of DAML commands.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlCommands {
    workflow_id: String,
    application_id: String,
    command_id: String,
    party: String,
    act_as: Vec<String>,
    read_as: Vec<String>,
    commands: Vec<DamlCommand>,
    deduplication_time: Option<Duration>,
    min_ledger_time: Option<DamlMinLedgerTime>,
}

impl DamlCommands {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workflow_id: impl Into<String>,
        application_id: impl Into<String>,
        command_id: impl Into<String>,
        party: impl Into<String>,
        act_as: impl Into<Vec<String>>,
        read_as: impl Into<Vec<String>>,
        commands: impl Into<Vec<DamlCommand>>,
        deduplication_time: impl Into<Option<Duration>>,
        min_ledger_time: impl Into<Option<DamlMinLedgerTime>>,
    ) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            application_id: application_id.into(),
            command_id: command_id.into(),
            party: party.into(),
            act_as: act_as.into(),
            read_as: read_as.into(),
            commands: commands.into(),
            deduplication_time: deduplication_time.into(),
            min_ledger_time: min_ledger_time.into(),
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

    pub fn act_as(&self) -> &[String] {
        &self.act_as
    }

    pub fn read_as(&self) -> &[String] {
        &self.read_as
    }

    pub fn commands(&self) -> &[DamlCommand] {
        &self.commands
    }

    pub const fn deduplication_time(&self) -> &Option<Duration> {
        &self.deduplication_time
    }

    pub const fn min_ledger_time(&self) -> &Option<DamlMinLedgerTime> {
        &self.min_ledger_time
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
            act_as: daml_commands.act_as,
            read_as: daml_commands.read_as,
            commands: daml_commands.commands.into_iter().map(Command::from).collect(),
            deduplication_time: daml_commands.deduplication_time.as_ref().map(util::to_grpc_duration).transpose()?,
            min_ledger_time_abs: match daml_commands.min_ledger_time {
                Some(DamlMinLedgerTime::Absolute(timestamp)) => Some(util::to_grpc_timestamp(timestamp)?),
                _ => None,
            },
            min_ledger_time_rel: match daml_commands.min_ledger_time {
                Some(DamlMinLedgerTime::Relative(duration)) => Some(util::to_grpc_duration(&duration)?),
                _ => None,
            },
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DamlMinLedgerTime {
    Absolute(DateTime<Utc>),
    Relative(Duration),
}
