use crate::data::identifier::DamlIdentifier;
use crate::data::value::DamlValue;
use crate::grpc_protobuf_autogen::commands::Command;
use crate::grpc_protobuf_autogen::commands::ExerciseCommand;

/// A command to exercise a choice on a contract on a DAML ledger.
#[derive(Debug, Eq, PartialEq)]
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

    pub fn template_id(&self) -> &DamlIdentifier {
        &self.template_id
    }

    pub fn contract_id(&self) -> &str {
        &self.contract_id
    }

    pub fn choice(&self) -> &str {
        &self.choice
    }

    pub fn choice_argument(&self) -> &DamlValue {
        &self.choice_argument
    }
}

impl From<DamlExerciseCommand> for Command {
    fn from(daml_exercise_command: DamlExerciseCommand) -> Self {
        let mut exercise_command = ExerciseCommand::new();
        exercise_command.set_template_id(daml_exercise_command.template_id.into());
        exercise_command.set_choice_argument(daml_exercise_command.choice_argument.into());
        exercise_command.set_contract_id(daml_exercise_command.contract_id);
        exercise_command.set_choice(daml_exercise_command.choice);
        let mut command = Self::new();
        command.set_exercise(exercise_command);
        command
    }
}
