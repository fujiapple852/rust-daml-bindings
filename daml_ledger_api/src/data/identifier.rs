use crate::grpc_protobuf_autogen::value::Identifier;

/// Unique identifier of an entity on a DAML ledger.
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct DamlIdentifier {
    package_id: String,
    module_name: String,
    entity_name: String,
}

impl DamlIdentifier {
    pub fn new(package_id: impl Into<String>, module_name: impl Into<String>, entity_name: impl Into<String>) -> Self {
        Self {
            package_id: package_id.into(),
            module_name: module_name.into(),
            entity_name: entity_name.into(),
        }
    }

    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn entity_name(&self) -> &str {
        &self.entity_name
    }
}

impl From<DamlIdentifier> for Identifier {
    fn from(daml_identifier: DamlIdentifier) -> Self {
        let mut identifier = Self::new();
        identifier.set_package_id(daml_identifier.package_id);
        identifier.set_module_name(daml_identifier.module_name);
        identifier.set_entity_name(daml_identifier.entity_name);
        identifier
    }
}

impl From<Identifier> for DamlIdentifier {
    fn from(mut id: Identifier) -> Self {
        Self::new(id.take_package_id(), id.take_module_name(), id.take_entity_name())
    }
}
