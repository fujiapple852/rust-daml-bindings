use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::{Date, DateTime, Utc};

use crate::data::DamlError;
use crate::nat::{Nat, Nat10};

/// Type alias for a DAML `Int`.
pub type DamlInt64 = i64;

/// Type alias for a DAML `Numeric`.
pub type DamlNumeric = BigDecimal;

/// Type alias for a Daml `Numeric 10`
pub type DamlNumeric10 = DamlFixedNumeric<Nat10>;

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

/// A fixed precision numeric type.  Currently a simple wrapper around a `BigDecimal`.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlFixedNumeric<T: Nat> {
    pub _phantom: PhantomData<T>,
    pub value: BigDecimal,
}

impl<T: Nat> DamlFixedNumeric<T> {
    pub fn new(value: BigDecimal) -> Self {
        Self {
            _phantom: PhantomData::<T>::default(),
            value,
        }
    }
    pub fn try_new(f: f64) -> Result<Self, DamlError> {
        Ok(Self::new(BigDecimal::try_from(f)?))
    }
}

/// Convert a f64 to a `DamlFixedNumeric`.
///
/// Note that this is not a fallible conversion and instead panics if the conversion fails.  Use
/// `DamlFixedNumeric::try_new` instead to construct a `DamlFixedNumerical` with returns an error on invalid input.
///
/// Arguable we could use the `TryFrom` trait here however the code generate currently produces entries such as
/// `my_numeric: impl Into<DamlNumeric10>` rather than `TryInto` which has the nice property of avoiding fallible cases.
#[allow(clippy::fallible_impl_from)]
impl<T: Nat> From<f64> for DamlFixedNumeric<T> {
    fn from(f: f64) -> Self {
        Self::new(match BigDecimal::try_from(f) {
            Ok(bd) => bd,
            Err(err) => panic!(format!("invalid f64: {}", err)),
        })
    }
}

impl<T: Nat> FromStr for DamlFixedNumeric<T> {
    type Err = DamlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(FromStr::from_str(s)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_try_new() {
        let num = DamlNumeric10::try_new(1.2_f64).unwrap();
        assert_eq!(DamlNumeric10::new(BigDecimal::try_from(1.2_f64).unwrap()), num);
    }

    #[test]
    fn test_numeric_try_new_nan() {
        let num = DamlNumeric10::try_new(1_f64 / 0_f64);
        assert!(num.is_err());
    }

    #[test]
    fn test_numeric_from() {
        let num = DamlNumeric10::from(1.2_f64);
        assert_eq!(DamlNumeric10::new(BigDecimal::try_from(1.2_f64).unwrap()), num);
    }

    #[test]
    #[should_panic]
    fn test_numeric_from_should_panic() {
        let _ = DamlNumeric10::from(1_f64 / 0_f64);
    }
}