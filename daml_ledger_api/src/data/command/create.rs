use crate::data::identifier::DamlIdentifier;
use crate::data::value::DamlRecord;
use crate::grpc_protobuf_autogen::commands::Command;
use crate::grpc_protobuf_autogen::commands::CreateCommand;

/// A command to create a contract on a DAML ledger.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlCreateCommand {
    template_id: DamlIdentifier,
    create_arguments: DamlRecord,
}

impl DamlCreateCommand {
    pub fn new(template_id: impl Into<DamlIdentifier>, create_arguments: impl Into<DamlRecord>) -> Self {
        Self {
            template_id: template_id.into(),
            create_arguments: create_arguments.into(),
        }
    }

    pub fn template_id(&self) -> &DamlIdentifier {
        &self.template_id
    }

    pub fn create_arguments(&self) -> &DamlRecord {
        &self.create_arguments
    }
}

impl From<DamlCreateCommand> for Command {
    fn from(daml_create_command: DamlCreateCommand) -> Self {
        let mut create_command = CreateCommand::new();
        create_command.set_template_id(daml_create_command.template_id.into());
        create_command.set_create_arguments(daml_create_command.create_arguments.into());
        let mut command = Self::new();
        command.set_create(create_command);
        command
    }
}
