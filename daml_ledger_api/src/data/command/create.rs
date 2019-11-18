use crate::data::identifier::DamlIdentifier;
use crate::data::value::DamlRecord;
use crate::grpc_protobuf_autogen::commands::Command;
use crate::grpc_protobuf_autogen::commands::CreateCommand;

/// Create a new contract instance based on a template.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlCreateCommand {
    pub template_id: DamlIdentifier,
    pub create_arguments: DamlRecord,
}

/// Create a new contract instance based on a template.
impl DamlCreateCommand {
    pub fn new(template_id: impl Into<DamlIdentifier>, create_arguments: impl Into<DamlRecord>) -> Self {
        Self {
            template_id: template_id.into(),
            create_arguments: create_arguments.into(),
        }
    }

    /// The template of contract the client wants to create.
    pub fn template_id(&self) -> &DamlIdentifier {
        &self.template_id
    }

    /// The arguments required for creating a contract from this template.
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
