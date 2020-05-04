use crate::data::identifier::DamlIdentifier;
use crate::data::value::DamlValue;
use crate::grpc_protobuf::com::daml::ledger::api::v1::command::Command;
use crate::grpc_protobuf::com::daml::ledger::api::v1::ExerciseCommand;

/// Exercise a choice on an existing contract.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlExerciseCommand {
    template_id: DamlIdentifier,
    contract_id: String,
    choice: String,
    choice_argument: DamlValue,
}

impl DamlExerciseCommand {
    pub fn new(
        template_id: impl Into<DamlIdentifier>,
        contract_id: impl Into<String>,
        choice: impl Into<String>,
        choice_argument: impl Into<DamlValue>,
    ) -> Self {
        Self {
            template_id: template_id.into(),
            contract_id: contract_id.into(),
            choice: choice.into(),
            choice_argument: choice_argument.into(),
        }
    }

    /// The template of contract the client wants to exercise.
    pub const fn template_id(&self) -> &DamlIdentifier {
        &self.template_id
    }

    /// The name of the choice the client wants to exercise.
    ///
    /// Must match the regexp ``[A-Za-z0-9#:\-_/ ]+``
    pub fn contract_id(&self) -> &str {
        &self.contract_id
    }

    /// The name of the choice the client wants to exercise.
    ///
    /// Must match the regexp `[A-Za-z\$_][A-Za-z0-9\$_]*`
    pub fn choice(&self) -> &str {
        &self.choice
    }

    /// The argument for this choice.
    pub const fn choice_argument(&self) -> &DamlValue {
        &self.choice_argument
    }
}

impl From<DamlExerciseCommand> for Command {
    fn from(daml_exercise_command: DamlExerciseCommand) -> Self {
        Command::Exercise(ExerciseCommand {
            template_id: Some(daml_exercise_command.template_id.into()),
            contract_id: daml_exercise_command.contract_id,
            choice: daml_exercise_command.choice,
            choice_argument: Some(daml_exercise_command.choice_argument.into()),
        })
    }
}
