use crate::data::identifier::DamlIdentifier;
use crate::data::value::DamlRecord;
use crate::grpc_protobuf::com::daml::ledger::api::v1::command::Command;
use crate::grpc_protobuf::com::daml::ledger::api::v1::CreateCommand;

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
        Command::Create(CreateCommand {
            template_id: Some(daml_create_command.template_id.into()),
            create_arguments: Some(daml_create_command.create_arguments.into()),
        })
    }
}
