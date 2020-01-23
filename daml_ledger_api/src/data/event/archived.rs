use crate::data::{DamlError, DamlIdentifier, DamlResult};
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::ArchivedEvent;
use crate::util::Required;
use std::convert::TryFrom;

/// An event which represents archiving a contract on a DAML ledger.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlArchivedEvent {
    pub event_id: String,
    pub contract_id: String,
    pub template_id: DamlIdentifier,
    pub witness_parties: Vec<String>,
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

impl TryFrom<ArchivedEvent> for DamlArchivedEvent {
    type Error = DamlError;

    fn try_from(e: ArchivedEvent) -> DamlResult<Self> {
        Ok(Self::new(e.event_id, e.contract_id, e.template_id.req().map(DamlIdentifier::from)?, e.witness_parties))
    }
}
