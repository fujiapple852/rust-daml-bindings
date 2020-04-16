use crate::data::value::{DamlRecord, DamlValue};
use crate::data::{DamlError, DamlIdentifier};
use crate::grpc_protobuf::com::daml::ledger::api::v1::CreatedEvent;
use crate::util::Required;
use std::convert::TryFrom;

/// An event which represents creating a contract on a DAML ledger.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlCreatedEvent {
    pub event_id: String,
    pub contract_id: String,
    pub template_id: DamlIdentifier,
    pub contract_key: Option<DamlValue>,
    pub create_arguments: DamlRecord,
    pub witness_parties: Vec<String>,
    pub signatories: Vec<String>,
    pub observers: Vec<String>,
    pub agreement_text: String,
}

/// Records that a contract has been created, and choices may now be exercised on it.
impl DamlCreatedEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_id: impl Into<String>,
        contract_id: impl Into<String>,
        template_id: impl Into<DamlIdentifier>,
        contract_key: impl Into<Option<DamlValue>>,
        create_arguments: impl Into<DamlRecord>,
        witness_parties: impl Into<Vec<String>>,
        signatories: impl Into<Vec<String>>,
        observers: impl Into<Vec<String>>,
        agreement_text: impl Into<String>,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            contract_id: contract_id.into(),
            template_id: template_id.into(),
            contract_key: contract_key.into(),
            create_arguments: create_arguments.into(),
            witness_parties: witness_parties.into(),
            signatories: signatories.into(),
            observers: observers.into(),
            agreement_text: agreement_text.into(),
        }
    }

    /// The ID of this particular event.
    ///
    /// Must match the regexp `[A-Za-z0-9#:\-_/ ]+`
    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    /// The ID of the created contract.
    ///
    /// Must match the regexp `[A-Za-z0-9#:\-_/ ]+`
    pub fn contract_id(&self) -> &str {
        &self.contract_id
    }

    /// The template of the created contract.
    pub fn template_id(&self) -> &DamlIdentifier {
        &self.template_id
    }

    /// The key of the created contract, if defined.
    pub fn contract_key(&self) -> &Option<DamlValue> {
        &self.contract_key
    }

    /// The arguments that have been used to create the contract.
    pub fn create_arguments(&self) -> &DamlRecord {
        &self.create_arguments
    }

    /// The parties that are notified of this event.
    ///
    /// For `CreatedEvent`s,  these are the intersection of the stakeholders of the contract in question and the
    /// parties specified in the `TransactionFilter`. The stakeholders are the union of the signatories and the
    /// observers of the contract.
    pub fn witness_parties(&self) -> &[String] {
        &self.witness_parties
    }

    /// The signatories for this contract as specified by the template.
    pub fn signatories(&self) -> &[String] {
        &self.signatories
    }

    /// The observers for this contract as specified explicitly by the template or implicitly as choice controllers.
    pub fn observers(&self) -> &[String] {
        &self.observers
    }

    /// The arguments that have been used to create the contract.
    pub fn take_create_arguments(self) -> DamlRecord {
        self.create_arguments
    }
}

impl TryFrom<CreatedEvent> for DamlCreatedEvent {
    type Error = DamlError;

    fn try_from(event: CreatedEvent) -> Result<Self, Self::Error> {
        Ok(Self::new(
            event.event_id,
            event.contract_id,
            event.template_id.req()?,
            event.contract_key.map(DamlValue::try_from).transpose()?,
            event.create_arguments.req().and_then(DamlRecord::try_from)?,
            event.witness_parties,
            event.signatories,
            event.observers,
            event.agreement_text.req()?,
        ))
    }
}
