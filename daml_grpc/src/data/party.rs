use crate::grpc_protobuf::com::daml::ledger::api::v1::admin::PartyDetails;

/// The details of a Daml party.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlPartyDetails {
    party: String,
    display_name: String,
    is_local: bool,
}

impl DamlPartyDetails {
    pub fn new(party: impl Into<String>, display_name: impl Into<String>, is_local: bool) -> Self {
        Self {
            party: party.into(),
            display_name: display_name.into(),
            is_local,
        }
    }

    /// The stable unique identifier of a Daml party.
    pub fn party(&self) -> &str {
        &self.party
    }

    /// Human readable name associated with the party. Caution, it might not be unique.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// true if party is hosted by the backing participant.
    pub const fn is_local(&self) -> bool {
        self.is_local
    }
}

impl From<PartyDetails> for DamlPartyDetails {
    fn from(details: PartyDetails) -> Self {
        Self::new(details.party, details.display_name, details.is_local)
    }
}
