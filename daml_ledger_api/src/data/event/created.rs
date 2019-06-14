use crate::data::identifier::DamlIdentifier;
use crate::data::value::DamlRecord;
use crate::data::DamlError;
use crate::grpc_protobuf_autogen::event::CreatedEvent;
use std::convert::{TryFrom, TryInto};

/// An event which represents creating a contract on a DAML ledger.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlCreatedEvent {
    event_id: String,
    contract_id: String,
    template_id: DamlIdentifier,
    create_arguments: DamlRecord,
    witness_parties: Vec<String>,
}

impl DamlCreatedEvent {
    pub fn new(
        event_id: impl Into<String>,
        contract_id: impl Into<String>,
        template_id: impl Into<DamlIdentifier>,
        create_arguments: impl Into<DamlRecord>,
        witness_parties: impl Into<Vec<String>>,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            contract_id: contract_id.into(),
            template_id: template_id.into(),
            create_arguments: create_arguments.into(),
            witness_parties: witness_parties.into(),
        }
    }

    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    pub fn contract_id(&self) -> &str {
        &self.contract_id
    }

    pub fn template_id(&self) -> &DamlIdentifier {
        &self.template_id
    }

    pub fn create_arguments(&self) -> &DamlRecord {
        &self.create_arguments
    }

    pub fn witness_parties(&self) -> &[String] {
        &self.witness_parties
    }

    pub fn take_create_arguments(self) -> DamlRecord {
        self.create_arguments
    }
}

impl TryFrom<CreatedEvent> for DamlCreatedEvent {
    type Error = DamlError;

    fn try_from(mut event: CreatedEvent) -> Result<Self, Self::Error> {
        let create_arguments: DamlRecord = event.take_create_arguments().try_into()?;
        Ok(Self::new(
            event.take_event_id(),
            event.take_contract_id(),
            event.take_template_id(),
            create_arguments,
            event.take_witness_parties(),
        ))
    }
}
