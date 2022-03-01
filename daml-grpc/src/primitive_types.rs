use crate::data::DamlError;
use crate::nat::{Nat, Nat10};
use bigdecimal::BigDecimal;
use chrono::{Date, DateTime, Utc};
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::fmt::Formatter;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

/// Type alias for a Daml `Int`.
pub type DamlInt64 = i64;

/// Type alias for a Daml `Numeric`.
pub type DamlNumeric = BigDecimal;

/// Type alias for a Daml `Numeric 10`
pub type DamlNumeric10 = DamlFixedNumeric<Nat10>;

/// Type alias for a Daml `Text`.
pub type DamlText = String;

/// Type alias for a Daml `Timestamp`.
pub type DamlTimestamp = DateTime<Utc>;

/// Type alias for a Daml `Bool`.
pub type DamlBool = bool;

/// Type alias for a Daml `Unit`.
pub type DamlUnit = ();

/// Type alias for a Daml `Date`.
pub type DamlDate = Date<Utc>;

/// Type alias for a Daml `List a`.
pub type DamlList<T> = Vec<T>;

/// Type alias for a Daml legacy `TextMap a b`.
pub type DamlTextMap<V> = DamlTextMapImpl<V>;

/// Type alias for a Daml `GenMap a b`.
pub type DamlGenMap<K, V> = BTreeMap<K, V>;

/// Type alias for a Daml `Optional a`.
pub type DamlOptional<T> = Option<T>;

/// A Daml `Party`.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
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

/// A Daml `ContractId`.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
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

/// A Daml legacy `TextMap a`.
#[derive(Debug, Eq, Default, Clone)]
pub struct DamlTextMapImpl<T>(pub HashMap<DamlText, T>);

impl<T> DamlTextMapImpl<T> {
    pub fn new() -> Self {
        DamlTextMapImpl(HashMap::new())
    }
}

/// Determine the order of `DamlTextMapImpl` objects.
///
/// The ordering of `DamlTextMapImpl` objects is determined by the number of entries and then by the ordering of keys,
/// values are not considered.
impl<T: PartialEq> PartialOrd for DamlTextMapImpl<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0.len() != other.0.len() {
            return self.0.len().partial_cmp(&other.0.len());
        }
        self.0.keys().sorted().partial_cmp(other.0.keys().sorted())
    }
}

impl<T: PartialOrd + Ord> Ord for DamlTextMapImpl<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("PartialOrd is never None")
    }
}

/// Compare `DamlTextMapImpl` for equality.
///
/// `DamlTextMapImpl` objects are considered equal if they contain the same number of entries and the same keys, values
/// are not considered.
impl<T> PartialEq for DamlTextMapImpl<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        self.0.keys().all(|key| other.get(key).is_some())
    }
}

impl<T> From<HashMap<DamlText, T>> for DamlTextMapImpl<T> {
    fn from(m: HashMap<DamlText, T>) -> Self {
        Self(m)
    }
}

impl<T> Deref for DamlTextMapImpl<T> {
    type Target = HashMap<DamlText, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for DamlTextMapImpl<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<V> IntoIterator for DamlTextMapImpl<V> {
    type IntoIter = <HashMap<DamlText, V> as IntoIterator>::IntoIter;
    type Item = (DamlText, V);

    fn into_iter(self) -> Self::IntoIter {
        HashMap::into_iter(self.0)
    }
}

impl<V> FromIterator<(DamlText, V)> for DamlTextMapImpl<V> {
    fn from_iter<T: IntoIterator<Item = (DamlText, V)>>(iter: T) -> DamlTextMapImpl<V> {
        Self::from(HashMap::from_iter(iter))
    }
}

/// A fixed precision numeric type.  Currently a simple wrapper around a `BigDecimal`.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
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
            Err(err) => panic!("invalid f64: {}", err),
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
        let _panic = DamlNumeric10::from(1_f64 / 0_f64);
    }

    #[test]
    fn test_daml_text_map_equal_keys() {
        let map1: DamlTextMapImpl<DamlInt64> =
            vec![("key1".into(), 10), ("key2".into(), 20)].into_iter().collect::<DamlTextMap<DamlInt64>>();
        let map2: DamlTextMap<DamlInt64> =
            vec![("key1".into(), 100), ("key2".into(), 200)].into_iter().collect::<DamlTextMap<DamlInt64>>();
        assert_eq!(map1, map2);
    }

    #[test]
    fn test_daml_text_map_not_equal_keys() {
        let map1: DamlTextMap<DamlInt64> =
            vec![("key1".into(), 10), ("key2".into(), 20)].into_iter().collect::<DamlTextMap<DamlInt64>>();
        let map2: DamlTextMap<DamlInt64> =
            vec![("key3".into(), 10), ("key4".into(), 20)].into_iter().collect::<DamlTextMap<DamlInt64>>();
        assert_ne!(map1, map2);
    }

    #[test]
    fn test_daml_text_map_order() {
        let map1: DamlTextMap<DamlInt64> =
            vec![("key1".into(), 10), ("key2".into(), 20)].into_iter().collect::<DamlTextMap<DamlInt64>>();
        let map2: DamlTextMap<DamlInt64> =
            vec![("key2".into(), 10), ("key3".into(), 20)].into_iter().collect::<DamlTextMap<DamlInt64>>();
        let map3: DamlTextMap<DamlInt64> =
            vec![("key1".into(), 100), ("key2".into(), 200)].into_iter().collect::<DamlTextMap<DamlInt64>>();
        assert_eq!(Ordering::Less, map1.cmp(&map2));
        assert_eq!(Ordering::Greater, map2.cmp(&map1));
        assert_eq!(Ordering::Equal, map1.cmp(&map3));
    }
}
