use bigdecimal::BigDecimal;
use chrono::{Date, DateTime, Utc};
use serde::export::Formatter;
use std::collections::HashMap;

/// Type alias for a DAML `Int`.
pub type DamlInt64 = i64;

/// Type alias for a DAML `Numeric`.
pub type DamlNumeric = BigDecimal;

/// Type alias for a DAML `Text`.
pub type DamlText = String;

/// Type alias for a DAML `Timestamp`.
pub type DamlTimestamp = DateTime<Utc>;

/// Type alias for a DAML `Bool`.
pub type DamlBool = bool;

/// Type alias for a DAML `Unit`.
pub type DamlUnit = ();

/// Type alias for a DAML `Date`.
pub type DamlDate = Date<Utc>;

/// Type alias for a DAML `List a`.
pub type DamlList<T> = Vec<T>;

/// Type alias for a DAML `TextMap a`.
pub type DamlTextMap<T> = HashMap<String, T>;

/// Type alias for a DAML `Optional a`.
pub type DamlOptional<T> = Option<T>;

/// A DAML `Party`.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlParty {
    pub party: String,
}

impl DamlParty {
    pub fn new(party: impl Into<String>) -> Self {
        Self {
            party: party.into(),
        }
    }

    pub fn as_str(&self) -> &str {
        self.party.as_str()
    }
}

impl From<&str> for DamlParty {
    fn from(party: &str) -> Self {
        DamlParty::new(party)
    }
}

impl From<String> for DamlParty {
    fn from(party: String) -> Self {
        DamlParty::new(party)
    }
}

impl PartialEq<&DamlParty> for &str {
    fn eq(&self, other: &&DamlParty) -> bool {
        *self == other.party
    }
}

impl PartialEq<&str> for &DamlParty {
    fn eq(&self, other: &&str) -> bool {
        self.party == *other
    }
}

impl PartialEq<DamlParty> for &str {
    fn eq(&self, other: &DamlParty) -> bool {
        self == &other.party
    }
}

impl PartialEq<&str> for DamlParty {
    fn eq(&self, other: &&str) -> bool {
        &self.party == other
    }
}

impl std::fmt::Display for DamlParty {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.party.fmt(f)
    }
}

/// A DAML `ContractId`.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlContractId {
    pub contract_id: String,
}

impl DamlContractId {
    pub fn new(contract_id: impl Into<String>) -> Self {
        Self {
            contract_id: contract_id.into(),
        }
    }

    pub fn as_str(&self) -> &str {
        self.contract_id.as_str()
    }
}

impl From<&str> for DamlContractId {
    fn from(contract_id: &str) -> Self {
        DamlContractId::new(contract_id)
    }
}

impl From<String> for DamlContractId {
    fn from(contract_id: String) -> Self {
        DamlContractId::new(contract_id)
    }
}

impl PartialEq<&DamlContractId> for &str {
    fn eq(&self, other: &&DamlContractId) -> bool {
        *self == other.contract_id
    }
}

impl PartialEq<&str> for &DamlContractId {
    fn eq(&self, other: &&str) -> bool {
        self.contract_id == *other
    }
}

impl PartialEq<DamlContractId> for &str {
    fn eq(&self, other: &DamlContractId) -> bool {
        self == &other.contract_id
    }
}

impl PartialEq<&str> for DamlContractId {
    fn eq(&self, other: &&str) -> bool {
        &self.contract_id == other
    }
}

impl std::fmt::Display for DamlContractId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.contract_id.fmt(f)
    }
}
