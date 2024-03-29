use crate::data::command::create::DamlCreateCommand;
use crate::data::command::exercise::DamlExerciseCommand;
use crate::data::command::exercise_by_key::DamlExerciseByKeyCommand;
use crate::data::command::DamlCreateAndExerciseCommand;
use crate::grpc_protobuf::com::daml::ledger::api::v1::Command;

/// A Daml ledger command.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DamlCommand {
    Create(DamlCreateCommand),
    Exercise(DamlExerciseCommand),
    ExerciseByKeyCommand(DamlExerciseByKeyCommand),
    CreateAndExercise(DamlCreateAndExerciseCommand),
}

impl From<DamlCommand> for Command {
    fn from(daml_command: DamlCommand) -> Self {
        Command {
            command: Some(match daml_command {
                DamlCommand::Create(c) => c.into(),
                DamlCommand::Exercise(c) => c.into(),
                DamlCommand::ExerciseByKeyCommand(c) => c.into(),
                DamlCommand::CreateAndExercise(c) => c.into(),
            }),
        }
    }
}
