use crate::grpc_protobuf::com::daml::ledger::api::v1::admin::PartyDetails;
use crate::util::is_default;

/// The details of a Daml party.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlPartyDetails {
    party: String,
    display_name: Option<String>,
    is_local: bool,
}

impl DamlPartyDetails {
    pub fn new<S: Into<String>>(party: impl Into<String>, display_name: Option<S>, is_local: bool) -> Self {
        Self {
            party: party.into(),
            display_name: display_name.map(Into::into),
            is_local,
        }
    }

    /// The stable unique identifier of a Daml party.
    pub fn party(&self) -> &str {
        &self.party
    }

    /// Human readable name associated with the party. Caution, it might not be unique.
    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    /// true if party is hosted by the backing participant.
    pub const fn is_local(&self) -> bool {
        self.is_local
    }
}

impl From<PartyDetails> for DamlPartyDetails {
    fn from(details: PartyDetails) -> Self {
        Self::new(details.party, Some(details.display_name).filter(|value| !is_default(value)), details.is_local)
    }
}
