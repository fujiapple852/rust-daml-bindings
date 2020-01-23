use crate::data::identifier::DamlIdentifier;
use crate::data::value::{DamlRecord, DamlValue};
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::command::Command;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::CreateAndExerciseCommand;

/// Create a contract and exercise a choice on it in the same transaction.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlCreateAndExerciseCommand {
    pub template_id: DamlIdentifier,
    pub create_arguments: DamlRecord,
    pub choice: String,
    pub choice_argument: DamlValue,
}

impl DamlCreateAndExerciseCommand {
    pub fn new(
        template_id: impl Into<DamlIdentifier>,
        create_arguments: impl Into<DamlRecord>,
        choice: impl Into<String>,
        choice_argument: impl Into<DamlValue>,
    ) -> Self {
        Self {
            template_id: template_id.into(),
            create_arguments: create_arguments.into(),
            choice: choice.into(),
            choice_argument: choice_argument.into(),
        }
    }

    /// The template of the contract the client wants to create.
    pub fn template_id(&self) -> &DamlIdentifier {
        &self.template_id
    }

    /// The arguments required for creating a contract from this template.
    pub fn create_arguments(&self) -> &DamlRecord {
        &self.create_arguments
    }

    /// The name of the choice the client wants to exercise.
    ///
    /// Must match the regexp `[A-Za-z\$_][A-Za-z0-9\$_]*`
    pub fn choice(&self) -> &str {
        &self.choice
    }

    /// The argument for this choice.
    pub fn choice_argument(&self) -> &DamlValue {
        &self.choice_argument
    }
}

impl From<DamlCreateAndExerciseCommand> for Command {
    fn from(daml_create_and_exercise_command: DamlCreateAndExerciseCommand) -> Self {
        Command::CreateAndExercise(CreateAndExerciseCommand {
            template_id: Some(daml_create_and_exercise_command.template_id.into()),
            create_arguments: Some(daml_create_and_exercise_command.create_arguments.into()),
            choice: daml_create_and_exercise_command.choice,
            choice_argument: Some(daml_create_and_exercise_command.choice_argument.into()),
        })
    }
}
