use crate::data::value::{DamlEnum, DamlRecord, DamlVariant};
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::daml::ledger::api::v1::value::Sum;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{
    gen_map, map, Enum, GenMap, List, Map, Optional, Record, Value, Variant,
};
use crate::util;
use crate::util::Required;
use std::convert::{TryFrom, TryInto};

use crate::nat::Nat;
use crate::primitive_types::{
    DamlBool, DamlContractId, DamlDate, DamlFixedNumeric, DamlGenMap, DamlInt64, DamlList, DamlNumeric, DamlOptional,
    DamlParty, DamlText, DamlTextMap, DamlTimestamp, DamlUnit,
};
use crate::serialize::{
    DamlDeserializableType, DamlDeserializeFrom, DamlDeserializeInto, DamlSerializableType, DamlSerializeFrom,
    DamlSerializeInto,
};
use itertools::Itertools;
use std::cmp::Ordering;
use std::str::FromStr;

/// A generic representation of data on a Daml ledger.
///
/// See the documentation for the Daml GRPC [Value](https://docs.daml.com/app-dev/grpc/proto-docs.html#value) type for
/// details.
///
/// The [`daml_value!`] macro can be used simplify the construction of complex [`DamlValue`] values.
///
/// [`daml_value!`]: https://docs.rs/daml-macro/0.2.2/daml_macro/macro.daml_value.html
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DamlValue {
    /// A Daml [`Record`](https://docs.daml.com/app-dev/grpc/proto-docs.html#record) value.
    Record(DamlRecord),
    /// A Daml [`Variant`](https://docs.daml.com/app-dev/grpc/proto-docs.html#variant) value.
    Variant(DamlVariant),
    /// A Daml [`Enum`](https://docs.daml.com/app-dev/grpc/proto-docs.html#enum) value.
    Enum(DamlEnum),
    /// A Daml `ContractId`.
    ContractId(DamlContractId),
    /// A Daml [`List`](https://docs.daml.com/app-dev/grpc/proto-docs.html#list) value.
    List(DamlList<DamlValue>),
    /// A Daml signed 64 bit integer value.
    Int64(DamlInt64),
    /// A Daml fixed precision numeric value.
    Numeric(DamlNumeric),
    /// A Daml text string value.
    Text(DamlText),
    /// A Daml timestamp value.
    Timestamp(DamlTimestamp),
    /// A Daml Party value.
    Party(DamlParty),
    /// A Daml boolean value.
    Bool(DamlBool),
    /// A Daml unit value.
    Unit,
    /// A Daml date value.
    Date(DamlDate),
    /// A Daml [optional value.
    Optional(Option<Box<DamlValue>>),
    /// A Daml [`Map`](https://docs.daml.com/app-dev/grpc/proto-docs.html#map) value.
    Map(DamlTextMap<DamlValue>),
    /// A Daml [`GenMap`](https://docs.daml.com/app-dev/grpc/proto-docs.html#genmap) value.
    GenMap(DamlGenMap<DamlValue, DamlValue>),
}

impl DamlValue {
    /// Construct a new [`DamlValue::Record`] from an existing [`DamlRecord`].
    pub fn new_record(record: impl Into<DamlRecord>) -> Self {
        DamlValue::Record(record.into())
    }

    /// Construct a new [`DamlValue::Variant`] from an existing [`DamlVariant`].
    pub fn new_variant(variant: impl Into<DamlVariant>) -> Self {
        DamlValue::Variant(variant.into())
    }

    /// Construct a new [`DamlValue::Enum`] from an existing [`DamlEnum`].
    pub fn new_enum(enum_variant: impl Into<DamlEnum>) -> Self {
        DamlValue::Enum(enum_variant.into())
    }

    /// Construct a new [`DamlValue::ContractId`] from an existing [`DamlContractId`].
    pub fn new_contract_id(contract_id: impl Into<DamlContractId>) -> Self {
        DamlValue::ContractId(contract_id.into())
    }

    /// Construct a new [`DamlValue::List`] from an existing [`DamlList<DamlValue>`].
    pub fn new_list(list: impl Into<DamlList<Self>>) -> Self {
        DamlValue::List(list.into())
    }

    /// Construct a new [`DamlValue::Int64`] from an existing [`DamlInt64`].
    pub fn new_int64(value: impl Into<DamlInt64>) -> Self {
        DamlValue::Int64(value.into())
    }

    /// Construct a new [`DamlValue::Numeric`] from an existing [`DamlNumeric`].
    pub fn new_numeric(numeric: impl Into<DamlNumeric>) -> Self {
        DamlValue::Numeric(numeric.into())
    }

    /// Construct a new [`DamlValue::Text`] from an existing [`DamlText`].
    pub fn new_text(text: impl Into<DamlText>) -> Self {
        DamlValue::Text(text.into())
    }

    /// Construct a new [`DamlValue::Timestamp`] from an existing [`DamlTimestamp`].
    pub fn new_timestamp(timestamp: impl Into<DamlTimestamp>) -> Self {
        DamlValue::Timestamp(timestamp.into())
    }

    /// Construct a new [`DamlValue::Party`] from an existing [`DamlParty`].
    pub fn new_party(party: impl Into<DamlParty>) -> Self {
        DamlValue::Party(party.into())
    }

    /// Construct a new [`DamlValue::Bool`] from an existing [`DamlBool`].
    pub fn new_bool(value: DamlBool) -> Self {
        DamlValue::Bool(value)
    }

    /// Construct a new [`DamlValue::Unit`].
    pub const fn new_unit() -> Self {
        DamlValue::Unit
    }

    /// Construct a new [`DamlValue::Date`] from an existing [`DamlDate`].
    pub fn new_date(date: impl Into<DamlDate>) -> Self {
        DamlValue::Date(date.into())
    }

    /// Construct a new [`DamlValue::Optional`] from an existing [`Option<DamlValue>`].
    pub fn new_optional(optional: Option<Self>) -> Self {
        DamlValue::Optional(optional.map(Box::new))
    }

    /// Construct a new [`DamlValue::Map`] from an existing [`DamlTextMap<DamlValue>`].
    pub fn new_map(map: impl Into<DamlTextMap<Self>>) -> Self {
        DamlValue::Map(map.into())
    }

    /// Construct a new [`DamlValue::GenMap`] from an existing [`DamlGenMap<DamlValue, DamlValue>`].
    pub fn new_genmap(map: impl Into<DamlGenMap<Self, Self>>) -> Self {
        DamlValue::GenMap(map.into())
    }

    /// Try to extract an `()` value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Unit`] then `()` is returned, otherwise a [`DamlError::UnexpectedType`] is
    /// returned.
    pub fn try_unit(&self) -> DamlResult<()> {
        match self {
            DamlValue::Unit => Ok(()),
            _ => Err(self.make_unexpected_type_error("Unit")),
        }
    }

    /// Try to extract an `&()` value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Unit`] then `&()` is returned, otherwise a [`DamlError::UnexpectedType`] is
    /// returned.
    pub fn try_unit_ref(&self) -> DamlResult<&()> {
        match self {
            DamlValue::Unit => Ok(&()),
            _ => Err(self.make_unexpected_type_error("Unit")),
        }
    }

    /// Try to extract an [`DamlDate`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Date`] then [`DamlDate`] is returned, otherwise a [`DamlError::UnexpectedType`] is
    /// returned.
    pub fn try_date(&self) -> DamlResult<DamlDate> {
        match *self {
            DamlValue::Date(d) => Ok(d),
            _ => Err(self.make_unexpected_type_error("Date")),
        }
    }

    /// Try to extract an &[`DamlDate`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Date`] then &[`DamlDate`] is returned, otherwise a [`DamlError::UnexpectedType`] is
    /// returned.
    pub fn try_date_ref(&self) -> DamlResult<&DamlDate> {
        match self {
            DamlValue::Date(d) => Ok(d),
            _ => Err(self.make_unexpected_type_error("Date")),
        }
    }

    /// Try to extract an [`DamlInt64`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Int64`] then [`DamlInt64`] is returned, otherwise a [`DamlError::UnexpectedType`] is
    /// returned.
    pub fn try_int64(&self) -> DamlResult<DamlInt64> {
        match self {
            DamlValue::Int64(i) => Ok(*i),
            _ => Err(self.make_unexpected_type_error("Int64")),
        }
    }

    /// Try to extract an &[`DamlInt64`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Int64`] then &[`DamlInt64`] is returned, otherwise a [`DamlError::UnexpectedType`]
    /// is returned.
    pub fn try_int64_ref(&self) -> DamlResult<&DamlInt64> {
        match self {
            DamlValue::Int64(i) => Ok(i),
            _ => Err(self.make_unexpected_type_error("Int64")),
        }
    }

    /// Try to extract an &[`DamlNumeric`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Numeric`] then &[`DamlNumeric`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_numeric(&self) -> DamlResult<&DamlNumeric> {
        match self {
            DamlValue::Numeric(d) => Ok(d),
            _ => Err(self.make_unexpected_type_error("Numeric")),
        }
    }

    /// Try to extract an [`DamlNumeric`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Numeric`] then a clone of the [`DamlNumeric`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_numeric_clone(&self) -> DamlResult<DamlNumeric> {
        match self {
            DamlValue::Numeric(d) => Ok(d.clone()),
            _ => Err(self.make_unexpected_type_error("Numeric")),
        }
    }

    /// Try to extract an [`DamlBool`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Bool`] then [`DamlBool`] is returned, otherwise a [`DamlError::UnexpectedType`] is
    /// returned.
    pub fn try_bool(&self) -> DamlResult<DamlBool> {
        match self {
            DamlValue::Bool(b) => Ok(*b),
            _ => Err(self.make_unexpected_type_error("Bool")),
        }
    }

    /// Try to extract an &[`DamlBool`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Bool`] then &[`DamlBool`] is returned, otherwise a [`DamlError::UnexpectedType`] is
    /// returned.
    pub fn try_bool_ref(&self) -> DamlResult<&DamlBool> {
        match self {
            DamlValue::Bool(b) => Ok(b),
            _ => Err(self.make_unexpected_type_error("Bool")),
        }
    }

    /// Try to extract an &[`DamlText`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Text`] then &[`DamlText`] is returned, otherwise a [`DamlError::UnexpectedType`] is
    /// returned.
    pub fn try_text(&self) -> DamlResult<&DamlText> {
        match self {
            DamlValue::Text(s) => Ok(s),
            _ => Err(self.make_unexpected_type_error("Text")),
        }
    }

    /// Try to extract an [`DamlTimestamp`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Timestamp`] then [`DamlTimestamp`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_timestamp(&self) -> DamlResult<DamlTimestamp> {
        match *self {
            DamlValue::Timestamp(ts) => Ok(ts),
            _ => Err(self.make_unexpected_type_error("Timestamp")),
        }
    }

    /// Try to extract an &[`DamlTimestamp`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Timestamp`] then &[`DamlTimestamp`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_timestamp_ref(&self) -> DamlResult<&DamlTimestamp> {
        match self {
            DamlValue::Timestamp(ts) => Ok(ts),
            _ => Err(self.make_unexpected_type_error("Timestamp")),
        }
    }

    /// Try to extract an &[`DamlParty`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Party`] then &[`DamlParty`] is returned, otherwise a [`DamlError::UnexpectedType`]
    /// is returned.
    pub fn try_party(&self) -> DamlResult<&DamlParty> {
        match self {
            DamlValue::Party(party) => Ok(party),
            _ => Err(self.make_unexpected_type_error("Party")),
        }
    }

    /// Try to extract an &[`DamlContractId`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::ContractId`] then &[`DamlContractId`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_contract_id(&self) -> DamlResult<&DamlContractId> {
        match self {
            DamlValue::ContractId(contract_id) => Ok(contract_id),
            _ => Err(self.make_unexpected_type_error("ContractId")),
        }
    }

    /// Try to extract an &[`DamlRecord`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Record`] then &[`DamlRecord`] is returned, otherwise a [`DamlError::UnexpectedType`]
    /// is returned.
    pub fn try_record(&self) -> DamlResult<&DamlRecord> {
        match self {
            DamlValue::Record(r) => Ok(r),
            _ => Err(self.make_unexpected_type_error("Record")),
        }
    }

    /// Try to extract an &[`DamlList<DamlValue>`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::List`] then &[`DamlList<DamlValue>`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_list(&self) -> DamlResult<&DamlList<Self>> {
        match self {
            DamlValue::List(l) => Ok(l),
            _ => Err(self.make_unexpected_type_error("List")),
        }
    }

    /// Try to extract an &[`DamlVariant`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Variant`] then &[`DamlVariant`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_variant(&self) -> DamlResult<&DamlVariant> {
        match self {
            DamlValue::Variant(v) => Ok(v),
            _ => Err(self.make_unexpected_type_error("Variant")),
        }
    }

    /// Try to extract an &[`DamlEnum`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Enum`] then &[`DamlEnum`] is returned, otherwise a [`DamlError::UnexpectedType`] is
    /// returned.
    pub fn try_enum(&self) -> DamlResult<&DamlEnum> {
        match self {
            DamlValue::Enum(e) => Ok(e),
            _ => Err(self.make_unexpected_type_error("Enum")),
        }
    }

    /// Try to extract an [`Option<&Self>`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Optional`] then [`Option<&Self>`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_optional(&self) -> DamlResult<Option<&Self>> {
        match self {
            DamlValue::Optional(opt) => Ok(opt.as_deref()),
            _ => Err(self.make_unexpected_type_error("Optional")),
        }
    }

    /// Try to extract an &[`DamlTextMap<DamlValue>`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Map`] then &[`DamlTextMap<DamlValue>`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_map(&self) -> DamlResult<&DamlTextMap<Self>> {
        match self {
            DamlValue::Map(m) => Ok(m),
            _ => Err(self.make_unexpected_type_error("Map")),
        }
    }

    /// Try to extract an &[`DamlGenMap<DamlValue, DamlValue>`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::GenMap`] then &[`DamlGenMap<DamlValue, DamlValue>`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_genmap(&self) -> DamlResult<&DamlGenMap<Self, Self>> {
        match self {
            DamlValue::GenMap(m) => Ok(m),
            _ => Err(self.make_unexpected_type_error("GenMap")),
        }
    }

    /// Try to take an [`DamlRecord`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Record`] then [`DamlRecord`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_take_record(self) -> DamlResult<DamlRecord> {
        match self {
            DamlValue::Record(r) => Ok(r),
            _ => Err(self.make_unexpected_type_error("Record")),
        }
    }

    /// Try to take an [`DamlVariant`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Variant`] then [`DamlVariant`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_take_variant(self) -> DamlResult<DamlVariant> {
        match self {
            DamlValue::Variant(v) => Ok(v),
            _ => Err(self.make_unexpected_type_error("Variant")),
        }
    }

    /// Try to take an [`DamlEnum`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Enum`] then [`DamlEnum`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_take_enum(self) -> DamlResult<DamlEnum> {
        match self {
            DamlValue::Enum(e) => Ok(e),
            _ => Err(self.make_unexpected_type_error("Enum")),
        }
    }

    /// Try to take an [`DamlList<DamlValue>`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::List`] then [`DamlList<DamlValue>`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_take_list(self) -> DamlResult<DamlList<Self>> {
        match self {
            DamlValue::List(l) => Ok(l),
            _ => Err(self.make_unexpected_type_error("List")),
        }
    }

    /// Try to take an [`DamlTextMap<DamlValue>`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Map`] then [`DamlTextMap<DamlValue>`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_take_map(self) -> DamlResult<DamlTextMap<Self>> {
        match self {
            DamlValue::Map(m) => Ok(m),
            _ => Err(self.make_unexpected_type_error("Map")),
        }
    }

    /// Try to take an [`DamlGenMap<DamlValue, DamlValue>`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::GenMap`] then [`DamlGenMap<DamlValue, DamlValue>`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_take_genmap(self) -> DamlResult<DamlGenMap<Self, Self>> {
        match self {
            DamlValue::GenMap(m) => Ok(m),
            _ => Err(self.make_unexpected_type_error("Map")),
        }
    }

    /// Try to take an [`Option<DamlValue>`] value from the [`DamlValue`].
    ///
    /// if `self` is a [`DamlValue::Optional`] then [`Option<DamlValue>`] is returned, otherwise a
    /// [`DamlError::UnexpectedType`] is returned.
    pub fn try_take_optional(self) -> DamlResult<Option<Self>> {
        match self {
            DamlValue::Optional(o) => Ok(o.map(|b| *b)),
            _ => Err(self.make_unexpected_type_error("Optional")),
        }
    }

    /// The name of this [`DamlValue`] variant type.
    pub fn variant_name(&self) -> &str {
        match self {
            DamlValue::Record(_) => "Record",
            DamlValue::Variant(_) => "Variant",
            DamlValue::Enum(_) => "Enum",
            DamlValue::ContractId(_) => "ContractId",
            DamlValue::List(_) => "List",
            DamlValue::Int64(_) => "Int64",
            DamlValue::Numeric(_) => "Numeric",
            DamlValue::Text(_) => "Text",
            DamlValue::Timestamp(_) => "Timestamp",
            DamlValue::Party(_) => "Party",
            DamlValue::Bool(_) => "Bool",
            DamlValue::Unit => "Unit",
            DamlValue::Date(_) => "Date",
            DamlValue::Optional(_) => "Optional",
            DamlValue::Map(_) => "Map",
            DamlValue::GenMap(_) => "GenMap",
        }
    }

    /// Apply a Daml data extractor function.
    ///
    /// A Daml data extractor function has the following signature:
    ///
    /// `Fn(&DamlRecord) -> DamlResult<R>`
    ///
    /// If this is [`DamlValue`] is a [`DamlValue::Record`] then the extractor function is applied, otherwise an
    /// [`DamlError::UnexpectedType`] error is returned.
    ///
    /// The extractor function can perform any arbitrary operation on the [`DamlRecord`] to produce an output value of
    /// any reference type or an error.  The intent is to provide a function which can extract data from the
    /// [`DamlRecord`] and any nested [`DamlValue`].
    ///
    /// This method is designed to be used with the `daml_path!` macro from the `daml-macro` crate which
    /// provides a concise DSL for constructing a Daml data extractor closure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use daml_grpc::data::value::{DamlRecord, DamlValue, DamlRecordField};
    /// # use daml_grpc::data::{DamlResult, DamlError};
    /// # use daml_grpc::data::DamlIdentifier;
    /// # use daml_grpc::primitive_types::{DamlParty, DamlText};
    /// # fn main() -> DamlResult<()> {
    /// let fields: Vec<DamlRecordField> = vec![DamlRecordField::new(Some("party"), DamlValue::new_party("Alice"))];
    /// let record: DamlRecord = DamlRecord::new(fields, None::<DamlIdentifier>);
    /// let record_value: DamlValue = DamlValue::new_record(record);
    /// let text_value: DamlValue = DamlValue::new_text("test");
    ///
    /// fn my_party_extractor(rec: &DamlRecord) -> DamlResult<&DamlParty> {
    ///     rec.field("party")?.try_party()
    /// }
    ///
    /// fn my_text_extractor(rec: &DamlRecord) -> DamlResult<&DamlText> {
    ///     rec.field("party")?.try_text()
    /// }
    ///
    /// assert_eq!("Alice", record_value.extract(my_party_extractor)?);
    /// assert_eq!(true, record_value.extract(my_text_extractor).is_err());
    /// assert_eq!(true, text_value.extract(my_party_extractor).is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn extract<'a, R, F>(&'a self, f: F) -> DamlResult<R>
    where
        F: FnOnce(&'a DamlRecord) -> DamlResult<R>,
    {
        f(self.try_record()?)
    }

    fn make_unexpected_type_error(&self, expected: &str) -> DamlError {
        DamlError::UnexpectedType(expected.to_owned(), self.variant_name().to_owned())
    }
}

impl From<()> for DamlValue {
    fn from(_: ()) -> Self {
        Self::new_unit()
    }
}

impl From<bool> for DamlValue {
    fn from(b: bool) -> Self {
        Self::new_bool(b)
    }
}

impl From<&str> for DamlValue {
    fn from(s: &str) -> Self {
        Self::new_text(s)
    }
}

impl From<String> for DamlValue {
    fn from(s: String) -> Self {
        Self::new_text(s)
    }
}

impl From<u8> for DamlValue {
    fn from(i: u8) -> Self {
        Self::new_int64(i)
    }
}

impl From<i8> for DamlValue {
    fn from(i: i8) -> Self {
        Self::new_int64(i)
    }
}

impl From<u16> for DamlValue {
    fn from(i: u16) -> Self {
        Self::new_int64(i)
    }
}

impl From<i16> for DamlValue {
    fn from(i: i16) -> Self {
        Self::new_int64(i)
    }
}

impl From<u32> for DamlValue {
    fn from(i: u32) -> Self {
        Self::new_int64(i)
    }
}

impl From<i32> for DamlValue {
    fn from(i: i32) -> Self {
        Self::new_int64(i)
    }
}

impl From<i64> for DamlValue {
    fn from(i: i64) -> Self {
        Self::new_int64(i)
    }
}
impl TryFrom<f32> for DamlValue {
    type Error = DamlError;

    fn try_from(d: f32) -> DamlResult<Self> {
        Ok(Self::new_numeric(DamlNumeric::try_from(d)?))
    }
}
impl TryFrom<f64> for DamlValue {
    type Error = DamlError;

    fn try_from(d: f64) -> DamlResult<Self> {
        Ok(Self::new_numeric(DamlNumeric::try_from(d)?))
    }
}

impl DamlSerializeFrom<DamlUnit> for DamlValue {
    fn serialize_from(_: DamlUnit) -> DamlValue {
        Self::new_unit()
    }
}

impl DamlSerializeFrom<DamlBool> for DamlValue {
    fn serialize_from(b: DamlBool) -> DamlValue {
        Self::new_bool(b)
    }
}

impl DamlSerializeFrom<DamlInt64> for DamlValue {
    fn serialize_from(i: DamlInt64) -> DamlValue {
        Self::new_int64(i)
    }
}

impl DamlSerializeFrom<DamlText> for DamlValue {
    fn serialize_from(text: DamlText) -> DamlValue {
        Self::new_text(text)
    }
}

impl DamlSerializeFrom<DamlParty> for DamlValue {
    fn serialize_from(party: DamlParty) -> DamlValue {
        Self::new_party(party.party)
    }
}

impl DamlSerializeFrom<DamlContractId> for DamlValue {
    fn serialize_from(contract_id: DamlContractId) -> DamlValue {
        Self::new_contract_id(contract_id.contract_id)
    }
}

impl<T> DamlSerializeFrom<DamlFixedNumeric<T>> for DamlValue
where
    T: DamlSerializableType + Nat,
{
    fn serialize_from(numeric: DamlFixedNumeric<T>) -> DamlValue {
        Self::new_numeric(numeric.value)
    }
}

impl DamlSerializeFrom<DamlTimestamp> for DamlValue {
    fn serialize_from(timestamp: DamlTimestamp) -> DamlValue {
        Self::new_timestamp(timestamp)
    }
}

impl DamlSerializeFrom<DamlDate> for DamlValue {
    fn serialize_from(date: DamlDate) -> DamlValue {
        Self::new_date(date)
    }
}

impl<T> DamlSerializeFrom<Box<T>> for DamlValue
where
    T: DamlSerializableType + DamlSerializeInto<DamlValue>,
{
    fn serialize_from(boxed: Box<T>) -> DamlValue {
        T::serialize_into(*boxed)
    }
}

impl<T> DamlSerializeFrom<DamlOptional<T>> for DamlValue
where
    T: DamlSerializableType + DamlSerializeInto<DamlValue>,
{
    fn serialize_from(optional: DamlOptional<T>) -> DamlValue {
        DamlValue::new_optional(optional.map(T::serialize_into))
    }
}

impl<T> DamlSerializeFrom<DamlList<T>> for DamlValue
where
    T: DamlSerializableType + DamlSerializeInto<DamlValue>,
{
    fn serialize_from(list: DamlList<T>) -> DamlValue {
        DamlValue::new_list(list.into_iter().map(T::serialize_into).collect::<DamlList<_>>())
    }
}

impl<K, V> DamlSerializeFrom<DamlGenMap<K, V>> for DamlValue
where
    K: DamlSerializableType + DamlSerializeInto<DamlValue>,
    V: DamlSerializableType + DamlSerializeInto<DamlValue>,
{
    fn serialize_from(gen_map: DamlGenMap<K, V>) -> DamlValue {
        DamlValue::new_genmap(
            gen_map
                .into_iter()
                .map(|(k, v)| (K::serialize_into(k), V::serialize_into(v)))
                .collect::<DamlGenMap<_, _>>(),
        )
    }
}

impl<V> DamlSerializeFrom<DamlTextMap<V>> for DamlValue
where
    V: DamlSerializableType + DamlSerializeInto<DamlValue>,
{
    fn serialize_from(text_map: DamlTextMap<V>) -> DamlValue {
        DamlValue::new_map(text_map.0.into_iter().map(|(k, v)| (k, V::serialize_into(v))).collect::<DamlTextMap<_>>())
    }
}

impl DamlDeserializeFrom for DamlUnit {
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::Unit => Ok(()),
            _ => Err(value.make_unexpected_type_error("Unit")),
        }
    }
}

impl DamlDeserializeFrom for DamlBool {
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::Bool(b) => Ok(b),
            _ => Err(value.make_unexpected_type_error("Bool")),
        }
    }
}

impl DamlDeserializeFrom for DamlInt64 {
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::Int64(i) => Ok(i),
            _ => Err(value.make_unexpected_type_error("Int64")),
        }
    }
}

impl DamlDeserializeFrom for DamlText {
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::Text(s) => Ok(s),
            _ => Err(value.make_unexpected_type_error("Text")),
        }
    }
}

impl DamlDeserializeFrom for DamlParty {
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::Party(party) => Ok(party),
            _ => Err(value.make_unexpected_type_error("Party")),
        }
    }
}

impl DamlDeserializeFrom for DamlContractId {
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::ContractId(contract_id) => Ok(contract_id),
            _ => Err(value.make_unexpected_type_error("ContractId")),
        }
    }
}

impl<T> DamlDeserializeFrom for DamlFixedNumeric<T>
where
    T: DamlDeserializableType + Nat,
{
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::Numeric(numeric) => Ok(DamlFixedNumeric::new(numeric)),
            _ => Err(value.make_unexpected_type_error("Numeric")),
        }
    }
}

impl DamlDeserializeFrom for DamlTimestamp {
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::Timestamp(timestamp) => Ok(timestamp),
            _ => Err(value.make_unexpected_type_error("Timestamp")),
        }
    }
}

impl DamlDeserializeFrom for DamlDate {
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::Date(date) => Ok(date),
            _ => Err(value.make_unexpected_type_error("Date")),
        }
    }
}

impl<T> DamlDeserializeFrom for Box<T>
where
    T: DamlDeserializeFrom + DamlDeserializableType,
{
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        Ok(Box::new(value.deserialize_into()?))
    }
}

impl<T> DamlDeserializeFrom for DamlOptional<T>
where
    T: DamlDeserializeFrom + DamlDeserializableType,
{
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::Optional(o) => Ok(o.map(|a| T::deserialize_from(*a)).transpose()?),
            _ => Err(value.make_unexpected_type_error("Option")),
        }
    }
}

impl<T> DamlDeserializeFrom for DamlList<T>
where
    T: DamlDeserializeFrom + DamlDeserializableType,
{
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::List(l) => Ok(l.into_iter().map(T::deserialize_from).collect::<DamlResult<DamlList<_>>>()?),
            _ => Err(value.make_unexpected_type_error("List")),
        }
    }
}

impl<V> DamlDeserializeFrom for DamlTextMap<V>
where
    V: DamlDeserializeFrom + DamlDeserializableType,
{
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::Map(text_map) => Ok(text_map
                .into_iter()
                .map(|(k, v)| Ok((k, V::deserialize_from(v)?)))
                .collect::<DamlResult<DamlTextMap<_>>>()?),
            _ => Err(value.make_unexpected_type_error("Map")),
        }
    }
}

impl<K, V> DamlDeserializeFrom for DamlGenMap<K, V>
where
    K: DamlDeserializeFrom + DamlDeserializableType + Ord,
    V: DamlDeserializeFrom + DamlDeserializableType,
{
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::GenMap(gen_map) => Ok(gen_map
                .into_iter()
                .map(|(k, v)| Ok((K::deserialize_from(k)?, V::deserialize_from(v)?)))
                .collect::<DamlResult<DamlGenMap<_, _>>>()?),
            _ => Err(value.make_unexpected_type_error("GenMap")),
        }
    }
}

impl TryFrom<Value> for DamlValue {
    type Error = DamlError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value.sum.req()? {
            Sum::Record(v) => DamlValue::Record(v.try_into()?),
            Sum::Variant(v) => DamlValue::Variant((*v).try_into()?),
            Sum::Enum(e) => DamlValue::Enum(e.into()),
            Sum::ContractId(v) => DamlValue::ContractId(DamlContractId::new(v)),
            Sum::List(v) =>
                DamlValue::List(v.elements.into_iter().map(TryInto::try_into).collect::<DamlResult<DamlList<_>>>()?),
            Sum::Int64(v) => DamlValue::Int64(v),
            Sum::Numeric(v) => DamlValue::Numeric(DamlNumeric::from_str(&v)?),
            Sum::Text(v) => DamlValue::Text(v),
            Sum::Timestamp(v) => DamlValue::Timestamp(util::datetime_from_micros(v)?),
            Sum::Party(v) => DamlValue::Party(DamlParty::new(v)),
            Sum::Bool(v) => DamlValue::Bool(v),
            Sum::Unit(_) => DamlValue::Unit,
            Sum::Date(v) => DamlValue::Date(util::date_from_days(v)?),
            Sum::Optional(v) =>
                DamlValue::Optional(v.value.map(|v| DamlValue::try_from(*v)).transpose()?.map(Box::new)),
            Sum::Map(v) => DamlValue::Map(
                v.entries
                    .into_iter()
                    .map(|v| Ok((v.key, v.value.req().and_then(DamlValue::try_from)?)))
                    .collect::<DamlResult<DamlTextMap<_>>>()?,
            ),
            Sum::GenMap(v) => DamlValue::GenMap(
                v.entries
                    .into_iter()
                    .map(|v| {
                        Ok((v.key.req().and_then(DamlValue::try_from)?, v.value.req().and_then(DamlValue::try_from)?))
                    })
                    .collect::<DamlResult<DamlGenMap<_, _>>>()?,
            ),
        })
    }
}

impl From<DamlValue> for Value {
    fn from(daml_value: DamlValue) -> Self {
        Self {
            sum: match daml_value {
                DamlValue::Record(v) => Some(Sum::Record(Record::from(v))),
                DamlValue::Variant(v) => Some(Sum::Variant(Box::new(Variant::from(v)))),
                DamlValue::Enum(e) => Some(Sum::Enum(Enum::from(e))),
                DamlValue::ContractId(v) => Some(Sum::ContractId(v.contract_id)),
                DamlValue::List(v) => Some(Sum::List(List {
                    elements: v.into_iter().map(Value::from).collect(),
                })),
                DamlValue::Int64(v) => Some(Sum::Int64(v)),
                // TODO: review the soundness of the numeric formatting here and consider using the `rust-decimal` crate
                DamlValue::Numeric(v) => Some(Sum::Numeric(format!("{:.37}", v))),
                DamlValue::Text(v) => Some(Sum::Text(v)), // value.set_text(v),
                DamlValue::Timestamp(v) => Some(Sum::Timestamp(v.timestamp())),
                DamlValue::Party(v) => Some(Sum::Party(v.party)),
                DamlValue::Bool(v) => Some(Sum::Bool(v)),
                DamlValue::Unit => Some(Sum::Unit(())),
                DamlValue::Date(v) => Some(Sum::Date(util::days_from_date(v))),
                DamlValue::Optional(Some(v)) => Some(Sum::Optional(Box::new(Optional {
                    value: Some(Box::new(Value::from(*v))),
                }))),
                DamlValue::Optional(None) => Some(Sum::Optional(Box::new(Optional {
                    value: None,
                }))),
                DamlValue::Map(v) => Some(Sum::Map(Map {
                    entries: v
                        .into_iter()
                        .map(|(key, val)| map::Entry {
                            key,
                            value: Some(val.into()),
                        })
                        .collect(),
                })),
                DamlValue::GenMap(v) => Some(Sum::GenMap(GenMap {
                    entries: v
                        .into_iter()
                        .map(|(key, val)| gen_map::Entry {
                            key: Some(key.into()),
                            value: Some(val.into()),
                        })
                        .collect(),
                })),
            },
        }
    }
}

/// A custom `PartialEq` implementation to allow us to order all possible `DamlValue` types.
///
/// This is required tp support the custom `Hash` implementation.
impl PartialOrd for DamlValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (DamlValue::Record(v1), DamlValue::Record(v2)) => v1.partial_cmp(v2),
            (DamlValue::Variant(v1), DamlValue::Variant(v2)) => v1.partial_cmp(v2),
            (DamlValue::Enum(v1), DamlValue::Enum(v2)) => v1.partial_cmp(v2),
            (DamlValue::List(v1), DamlValue::List(v2)) => v1.partial_cmp(v2.as_ref()),
            (DamlValue::Int64(v1), DamlValue::Int64(v2)) => v1.partial_cmp(v2),
            (DamlValue::Numeric(v1), DamlValue::Numeric(v2)) => v1.partial_cmp(v2),
            (DamlValue::Text(v1), DamlValue::Text(v2)) => v1.partial_cmp(v2),
            (DamlValue::Party(v1), DamlValue::Party(v2)) => v1.party.partial_cmp(&v2.party),
            (DamlValue::ContractId(v1), DamlValue::ContractId(v2)) => v1.contract_id.partial_cmp(&v2.contract_id),
            (DamlValue::Timestamp(v1), DamlValue::Timestamp(v2)) => v1.partial_cmp(v2),
            (DamlValue::Bool(v1), DamlValue::Bool(v2)) => v1.partial_cmp(v2),
            (DamlValue::Unit, DamlValue::Unit) => Some(Ordering::Equal),
            (DamlValue::Date(v1), DamlValue::Date(v2)) => v1.partial_cmp(v2),
            (DamlValue::Optional(v1), DamlValue::Optional(v2)) => v1.partial_cmp(v2),
            (DamlValue::Map(v1), DamlValue::Map(v2)) =>
                if v1.len() == v2.len() {
                    v1.keys().sorted().partial_cmp(v2.keys().sorted())
                } else {
                    v1.len().partial_cmp(&v2.len())
                },
            (DamlValue::GenMap(v1), DamlValue::GenMap(v2)) =>
                if v1.len() == v2.len() {
                    v1.keys().partial_cmp(v2.keys())
                } else {
                    v1.len().partial_cmp(&v2.len())
                },
            _ => None,
        }
    }
}

impl Ord for DamlValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::value::DamlValue;
    use crate::primitive_types::{DamlGenMap, DamlTextMap};
    use std::cmp::Ordering;

    #[test]
    fn test_eq_for_text() {
        let value1 = DamlValue::Text(String::from("text"));
        let value2 = DamlValue::Text(String::from("text"));
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_ord_for_text() {
        let value1 = DamlValue::Text(String::from("textA"));
        let value2 = DamlValue::Text(String::from("textB"));
        assert_eq!(value1.cmp(&value2), Ordering::Less);
    }

    #[test]
    fn test_eq_for_i64() {
        let value1 = DamlValue::Int64(101);
        let value2 = DamlValue::Int64(101);
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_ord_for_i64() {
        let value1 = DamlValue::Int64(102);
        let value2 = DamlValue::Int64(101);
        assert_eq!(value1.cmp(&value2), Ordering::Greater);
    }

    #[test]
    fn test_eq_for_textmap() {
        let items =
            vec![(String::from("text1"), DamlValue::Int64(100)), (String::from("text2"), DamlValue::Int64(200))];
        let value1 = DamlValue::Map(items.clone().into_iter().collect::<DamlTextMap<DamlValue>>());
        let value2 = DamlValue::Map(items.into_iter().collect::<DamlTextMap<DamlValue>>());
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_ord_for_textmap() {
        let items1 =
            vec![(String::from("text1"), DamlValue::Int64(100)), (String::from("text2"), DamlValue::Int64(200))];
        let items2 =
            vec![(String::from("text2"), DamlValue::Int64(100)), (String::from("text3"), DamlValue::Int64(200))];
        let items3 =
            vec![(String::from("text1"), DamlValue::Int64(100)), (String::from("text2"), DamlValue::Int64(200))];
        let value1 = DamlValue::Map(items1.into_iter().collect::<DamlTextMap<DamlValue>>());
        let value2 = DamlValue::Map(items2.into_iter().collect::<DamlTextMap<DamlValue>>());
        let value3 = DamlValue::Map(items3.into_iter().collect::<DamlTextMap<DamlValue>>());
        assert_eq!(value1.cmp(&value2), Ordering::Less);
        assert_eq!(value2.cmp(&value1), Ordering::Greater);
        assert_eq!(value1.cmp(&value3), Ordering::Equal);
    }

    #[test]
    fn test_eq_for_genmap() {
        let items = vec![
            (DamlValue::Text(String::from("text1")), DamlValue::Int64(100)),
            (DamlValue::Text(String::from("text2")), DamlValue::Int64(200)),
        ];
        let value1 = DamlValue::GenMap(items.clone().into_iter().collect::<DamlGenMap<_, _>>());
        let value2 = DamlValue::GenMap(items.into_iter().collect::<DamlGenMap<_, _>>());
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_ord_for_genmap() {
        let items1 = vec![
            (DamlValue::Text(String::from("text1")), DamlValue::Int64(100)),
            (DamlValue::Text(String::from("text2")), DamlValue::Int64(200)),
        ];
        let items2 = vec![
            (DamlValue::Text(String::from("text2")), DamlValue::Int64(100)),
            (DamlValue::Text(String::from("text3")), DamlValue::Int64(200)),
        ];
        let items3 = vec![
            (DamlValue::Text(String::from("text1")), DamlValue::Int64(100)),
            (DamlValue::Text(String::from("text2")), DamlValue::Int64(200)),
        ];
        let value1 = DamlValue::GenMap(items1.into_iter().collect::<DamlGenMap<DamlValue, DamlValue>>());
        let value2 = DamlValue::GenMap(items2.into_iter().collect::<DamlGenMap<DamlValue, DamlValue>>());
        let value3 = DamlValue::GenMap(items3.into_iter().collect::<DamlGenMap<DamlValue, DamlValue>>());
        assert_eq!(value1.cmp(&value2), Ordering::Less);
        assert_eq!(value2.cmp(&value1), Ordering::Greater);
        assert_eq!(value1.cmp(&value3), Ordering::Equal);
    }
}
