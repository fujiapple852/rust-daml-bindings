use crate::grpc_protobuf_autogen::party_management_service::PartyDetails;

/// The details of a DAML party.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlPartyDetails {
    pub party: String,
    pub display_name: String,
    pub is_local: bool,
}

impl DamlPartyDetails {
    pub fn new(party: impl Into<String>, display_name: impl Into<String>, is_local: bool) -> Self {
        Self {
            party: party.into(),
            display_name: display_name.into(),
            is_local,
        }
    }

    /// The stable unique identifier of a DAML party.
    pub fn party(&self) -> &str {
        &self.party
    }

    /// Human readable name associated with the party. Caution, it might not be unique.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// true if party is hosted by the backing participant.
    pub fn is_local(&self) -> bool {
        self.is_local
    }
}

impl From<PartyDetails> for DamlPartyDetails {
    fn from(mut details: PartyDetails) -> Self {
        Self::new(details.take_party(), details.take_display_name(), details.is_local)
    }
}
