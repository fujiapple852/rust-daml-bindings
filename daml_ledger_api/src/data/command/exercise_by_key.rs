use crate::data::identifier::DamlIdentifier;
use crate::data::value::DamlValue;
use crate::grpc_protobuf_autogen::commands::Command;
use crate::grpc_protobuf_autogen::commands::ExerciseByKeyCommand;

/// Exercise a choice on an existing contract specified by its key.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlExerciseByKeyCommand {
    pub template_id: DamlIdentifier,
    pub contract_key: DamlValue,
    pub choice: String,
    pub choice_argument: DamlValue,
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
    pub fn template_id(&self) -> &DamlIdentifier {
        &self.template_id
    }

    /// The key of the contract the client wants to exercise upon.
    pub fn contract_key(&self) -> &DamlValue {
        &self.contract_key
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

impl From<DamlExerciseByKeyCommand> for Command {
    fn from(daml_exercise_command: DamlExerciseByKeyCommand) -> Self {
        let mut exercise_by_key_command = ExerciseByKeyCommand::new();
        exercise_by_key_command.set_template_id(daml_exercise_command.template_id.into());
        exercise_by_key_command.set_choice_argument(daml_exercise_command.choice_argument.into());
        exercise_by_key_command.set_contract_key(daml_exercise_command.contract_key.into());
        exercise_by_key_command.set_choice(daml_exercise_command.choice);
        let mut command = Self::new();
        command.set_exerciseByKey(exercise_by_key_command);
        command
    }
}
