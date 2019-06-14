use crate::data::identifier::DamlIdentifier;
use crate::grpc_protobuf_autogen::event::ArchivedEvent;

/// An event which represents archiving a contract on a DAML ledger.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlArchivedEvent {
    event_id: String,
    contract_id: String,
    template_id: DamlIdentifier,
    witness_parties: Vec<String>,
}

impl DamlArchivedEvent {
    pub fn new(
        event_id: impl Into<String>,
        contract_id: impl Into<String>,
        template_id: impl Into<DamlIdentifier>,
        witness_parties: impl Into<Vec<String>>,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            contract_id: contract_id.into(),
            template_id: template_id.into(),
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

    pub fn witness_parties(&self) -> &[String] {
        &self.witness_parties
    }
}

impl From<ArchivedEvent> for DamlArchivedEvent {
    fn from(mut e: ArchivedEvent) -> Self {
        Self::new(e.take_event_id(), e.take_contract_id(), e.take_template_id(), e.take_witness_parties())
    }
}
