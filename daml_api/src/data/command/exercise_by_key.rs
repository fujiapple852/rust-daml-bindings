use crate::data::identifier::DamlIdentifier;
use crate::data::value::DamlValue;
use crate::grpc_protobuf::com::daml::ledger::api::v1::command::Command;
use crate::grpc_protobuf::com::daml::ledger::api::v1::ExerciseByKeyCommand;

/// Exercise a choice on an existing contract specified by its key.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlExerciseByKeyCommand {
    template_id: DamlIdentifier,
    contract_key: DamlValue,
    choice: String,
    choice_argument: DamlValue,
}

impl DamlExerciseByKeyCommand {
    pub fn new(
        template_id: impl Into<DamlIdentifier>,
        contract_key: impl Into<DamlValue>,
        choice: impl Into<String>,
        choice_argument: impl Into<DamlValue>,
    ) -> Self {
        Self {
            template_id: template_id.into(),
            contract_key: contract_key.into(),
            choice: choice.into(),
            choice_argument: choice_argument.into(),
        }
    }

    /// The template of contract the client wants to exercise.
    pub const fn template_id(&self) -> &DamlIdentifier {
        &self.template_id
    }

    /// The key of the contract the client wants to exercise upon.
    pub const fn contract_key(&self) -> &DamlValue {
        &self.contract_key
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

impl From<DamlExerciseByKeyCommand> for Command {
    fn from(daml_exercise_command: DamlExerciseByKeyCommand) -> Self {
        Command::ExerciseByKey(ExerciseByKeyCommand {
            template_id: Some(daml_exercise_command.template_id.into()),
            contract_key: Some(daml_exercise_command.contract_key.into()),
            choice: daml_exercise_command.choice,
            choice_argument: Some(daml_exercise_command.choice_argument.into()),
        })
    }
}
