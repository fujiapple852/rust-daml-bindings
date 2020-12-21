use crate::data::value::{DamlEnum, DamlRecord, DamlVariant};
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::daml::ledger::api::v1::value::Sum;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{
    gen_map, map, Enum, GenMap, List, Map, Optional, Record, Value, Variant,
};
use crate::util;
use crate::util::Required;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use crate::nat::Nat;
use crate::primitive_types::{
    DamlBool, DamlContractId, DamlDate, DamlFixedNumeric, DamlInt64, DamlNumeric, DamlParty, DamlText, DamlTimestamp,
    DamlUnit,
};
use crate::serialize::{
    DamlDeserializableType, DamlDeserializeFrom, DamlDeserializeInto, DamlSerializableType, DamlSerializeFrom,
    DamlSerializeInto,
};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// A generic representation of data on a DAML ledger.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DamlValue {
    Record(DamlRecord),
    Variant(DamlVariant),
    Enum(DamlEnum),
    ContractId(DamlContractId),
    List(Vec<DamlValue>),
    Int64(DamlInt64),
    Numeric(DamlNumeric),
    Text(DamlText),
    Timestamp(DamlTimestamp),
    Party(DamlParty),
    Bool(DamlBool),
    Unit,
    Date(DamlDate),
    Optional(Option<Box<DamlValue>>),
    Map(HashMap<String, DamlValue>),
    GenMap(HashMap<DamlValue, DamlValue>),
}

impl DamlValue {
    pub fn new_record(record: impl Into<DamlRecord>) -> Self {
        DamlValue::Record(record.into())
    }

    pub fn new_variant(variant: impl Into<DamlVariant>) -> Self {
        DamlValue::Variant(variant.into())
    }

    pub fn new_enum(enum_variant: impl Into<DamlEnum>) -> Self {
        DamlValue::Enum(enum_variant.into())
    }

    pub fn new_contract_id(contract_id: impl Into<DamlContractId>) -> Self {
        DamlValue::ContractId(contract_id.into())
    }

    pub fn new_list(list: impl Into<Vec<Self>>) -> Self {
        DamlValue::List(list.into())
    }

    pub fn new_int64(value: impl Into<DamlInt64>) -> Self {
        DamlValue::Int64(value.into())
    }

    pub fn new_numeric(numeric: impl Into<DamlNumeric>) -> Self {
        DamlValue::Numeric(numeric.into())
    }

    pub fn new_text(text: impl Into<DamlText>) -> Self {
        DamlValue::Text(text.into())
    }

    pub fn new_timestamp(timestamp: impl Into<DamlTimestamp>) -> Self {
        DamlValue::Timestamp(timestamp.into())
    }

    pub fn new_party(party: impl Into<DamlParty>) -> Self {
        DamlValue::Party(party.into())
    }

    pub fn new_bool(value: DamlBool) -> Self {
        DamlValue::Bool(value)
    }

    pub const fn new_unit() -> Self {
        DamlValue::Unit
    }

    pub fn new_date(date: impl Into<DamlDate>) -> Self {
        DamlValue::Date(date.into())
    }

    pub fn new_optional(optional: Option<Self>) -> Self {
        DamlValue::Optional(optional.map(Box::new))
    }

    pub fn new_map(map: impl Into<HashMap<String, Self>>) -> Self {
        DamlValue::Map(map.into())
    }

    pub fn new_genmap(map: impl Into<HashMap<Self, Self>>) -> Self {
        DamlValue::GenMap(map.into())
    }

    pub fn try_unit(&self) -> DamlResult<()> {
        match self {
            DamlValue::Unit => Ok(()),
            _ => Err(self.make_unexpected_type_error("Unit")),
        }
    }

    pub fn try_unit_ref(&self) -> DamlResult<&()> {
        match self {
            DamlValue::Unit => Ok(&()),
            _ => Err(self.make_unexpected_type_error("Unit")),
        }
    }

    pub fn try_date(&self) -> DamlResult<DamlDate> {
        match *self {
            DamlValue::Date(d) => Ok(d),
            _ => Err(self.make_unexpected_type_error("Date")),
        }
    }

    pub fn try_date_ref(&self) -> DamlResult<&DamlDate> {
        match self {
            DamlValue::Date(d) => Ok(d),
            _ => Err(self.make_unexpected_type_error("Date")),
        }
    }

    pub fn try_int64(&self) -> DamlResult<DamlInt64> {
        match self {
            DamlValue::Int64(i) => Ok(*i),
            _ => Err(self.make_unexpected_type_error("Int64")),
        }
    }

    pub fn try_int64_ref(&self) -> DamlResult<&DamlInt64> {
        match self {
            DamlValue::Int64(i) => Ok(i),
            _ => Err(self.make_unexpected_type_error("Int64")),
        }
    }

    pub fn try_numeric(&self) -> DamlResult<&DamlNumeric> {
        match self {
            DamlValue::Numeric(d) => Ok(d),
            _ => Err(self.make_unexpected_type_error("Numeric")),
        }
    }

    // BigDecimal does not implement the Copy trait
    pub fn try_numeric_clone(&self) -> DamlResult<DamlNumeric> {
        match self {
            DamlValue::Numeric(d) => Ok(d.clone()),
            _ => Err(self.make_unexpected_type_error("Numeric")),
        }
    }

    pub fn try_bool(&self) -> DamlResult<DamlBool> {
        match self {
            DamlValue::Bool(b) => Ok(*b),
            _ => Err(self.make_unexpected_type_error("Bool")),
        }
    }

    pub fn try_bool_ref(&self) -> DamlResult<&DamlBool> {
        match self {
            DamlValue::Bool(b) => Ok(b),
            _ => Err(self.make_unexpected_type_error("Bool")),
        }
    }

    pub fn try_text(&self) -> DamlResult<&DamlText> {
        match self {
            DamlValue::Text(s) => Ok(s),
            _ => Err(self.make_unexpected_type_error("Text")),
        }
    }

    pub fn try_timestamp(&self) -> DamlResult<DamlTimestamp> {
        match *self {
            DamlValue::Timestamp(ts) => Ok(ts),
            _ => Err(self.make_unexpected_type_error("Timestamp")),
        }
    }

    pub fn try_timestamp_ref(&self) -> DamlResult<&DamlTimestamp> {
        match self {
            DamlValue::Timestamp(ts) => Ok(ts),
            _ => Err(self.make_unexpected_type_error("Timestamp")),
        }
    }

    pub fn try_party(&self) -> DamlResult<&DamlParty> {
        match self {
            DamlValue::Party(party) => Ok(party),
            _ => Err(self.make_unexpected_type_error("Party")),
        }
    }

    pub fn try_contract_id(&self) -> DamlResult<&DamlContractId> {
        match self {
            DamlValue::ContractId(contract_id) => Ok(contract_id),
            _ => Err(self.make_unexpected_type_error("ContractId")),
        }
    }

    pub fn try_record(&self) -> DamlResult<&DamlRecord> {
        match self {
            DamlValue::Record(r) => Ok(r),
            _ => Err(self.make_unexpected_type_error("Record")),
        }
    }

    pub fn try_list(&self) -> DamlResult<&Vec<Self>> {
        match self {
            DamlValue::List(l) => Ok(l),
            _ => Err(self.make_unexpected_type_error("List")),
        }
    }

    pub fn try_variant(&self) -> DamlResult<&DamlVariant> {
        match self {
            DamlValue::Variant(v) => Ok(v),
            _ => Err(self.make_unexpected_type_error("Variant")),
        }
    }

    pub fn try_enum(&self) -> DamlResult<&DamlEnum> {
        match self {
            DamlValue::Enum(e) => Ok(e),
            _ => Err(self.make_unexpected_type_error("Enum")),
        }
    }

    pub fn try_optional(&self) -> DamlResult<Option<&Self>> {
        match self {
            DamlValue::Optional(opt) => Ok(opt.as_deref()),
            _ => Err(self.make_unexpected_type_error("Optional")),
        }
    }

    pub fn try_map(&self) -> DamlResult<&HashMap<String, Self>> {
        match self {
            DamlValue::Map(m) => Ok(m),
            _ => Err(self.make_unexpected_type_error("Map")),
        }
    }

    pub fn try_genmap(&self) -> DamlResult<&HashMap<Self, Self>> {
        match self {
            DamlValue::GenMap(m) => Ok(m),
            _ => Err(self.make_unexpected_type_error("GenMap")),
        }
    }

    pub fn try_take_record(self) -> DamlResult<DamlRecord> {
        match self {
            DamlValue::Record(r) => Ok(r),
            _ => Err(self.make_unexpected_type_error("Record")),
        }
    }

    pub fn try_take_variant(self) -> DamlResult<DamlVariant> {
        match self {
            DamlValue::Variant(v) => Ok(v),
            _ => Err(self.make_unexpected_type_error("Variant")),
        }
    }

    pub fn try_take_enum(self) -> DamlResult<DamlEnum> {
        match self {
            DamlValue::Enum(e) => Ok(e),
            _ => Err(self.make_unexpected_type_error("Enum")),
        }
    }

    pub fn try_take_list(self) -> DamlResult<Vec<Self>> {
        match self {
            DamlValue::List(l) => Ok(l),
            _ => Err(self.make_unexpected_type_error("List")),
        }
    }

    pub fn try_take_map(self) -> DamlResult<HashMap<String, Self>> {
        match self {
            DamlValue::Map(m) => Ok(m),
            _ => Err(self.make_unexpected_type_error("Map")),
        }
    }

    pub fn try_take_genmap(self) -> DamlResult<HashMap<Self, Self>> {
        match self {
            DamlValue::GenMap(m) => Ok(m),
            _ => Err(self.make_unexpected_type_error("Map")),
        }
    }

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

    /// Apply a DAML data extractor function.
    ///
    /// A DAML data extractor function has the following signature:
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
    /// This method is designed to be used with the `daml_path!` macro from the `daml_macro` crate which
    /// provides a concise DSL for constructing a DAML data extractor closure.
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
impl From<f32> for DamlValue {
    fn from(d: f32) -> Self {
        Self::new_numeric(d)
    }
}
impl From<f64> for DamlValue {
    fn from(d: f64) -> Self {
        Self::new_numeric(d)
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

impl<T> DamlSerializeFrom<Option<T>> for DamlValue
where
    T: DamlSerializableType + DamlSerializeInto<DamlValue>,
{
    fn serialize_from(optional: Option<T>) -> DamlValue {
        DamlValue::new_optional(optional.map(T::serialize_into))
    }
}

impl<T> DamlSerializeFrom<Vec<T>> for DamlValue
where
    T: DamlSerializableType + DamlSerializeInto<DamlValue>,
{
    fn serialize_from(list: Vec<T>) -> DamlValue {
        DamlValue::new_list(list.into_iter().map(T::serialize_into).collect::<Vec<_>>())
    }
}

#[allow(clippy::implicit_hasher)]
impl<T> DamlSerializeFrom<HashMap<String, T>> for DamlValue
where
    T: DamlSerializableType + DamlSerializeInto<DamlValue>,
{
    fn serialize_from(text_map: HashMap<String, T>) -> DamlValue {
        DamlValue::new_map(text_map.into_iter().map(|(k, v)| (k, T::serialize_into(v))).collect::<HashMap<String, _>>())
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

impl<T> DamlDeserializeFrom for Option<T>
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

impl<T> DamlDeserializeFrom for Vec<T>
where
    T: DamlDeserializeFrom + DamlDeserializableType,
{
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::List(l) => Ok(l.into_iter().map(T::deserialize_from).collect::<DamlResult<Vec<_>>>()?),
            _ => Err(value.make_unexpected_type_error("List")),
        }
    }
}

#[allow(clippy::implicit_hasher)]
impl<T> DamlDeserializeFrom for HashMap<String, T>
where
    T: DamlDeserializeFrom + DamlDeserializableType,
{
    fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
        match value {
            DamlValue::Map(text_map) => Ok(text_map
                .into_iter()
                .map(|(k, v)| Ok((k, T::deserialize_from(v)?)))
                .collect::<DamlResult<HashMap<String, _>>>()?),
            _ => Err(value.make_unexpected_type_error("Map")),
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
                DamlValue::List(v.elements.into_iter().map(TryInto::try_into).collect::<DamlResult<Vec<_>>>()?),
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
                    .collect::<DamlResult<HashMap<_, _>>>()?,
            ),
            Sum::GenMap(v) => DamlValue::GenMap(
                v.entries
                    .into_iter()
                    .map(|v| {
                        Ok((v.key.req().and_then(DamlValue::try_from)?, v.value.req().and_then(DamlValue::try_from)?))
                    })
                    .collect::<DamlResult<HashMap<_, _>>>()?,
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

/// A custom implementation of `Hash` for `DamlValue`.
///
/// The DAML ledger API allows any arbitrary `DamlValue` to be the key to a `DamlValue::GenMap`and so this requires
/// that we be able to derive a stable hash for all possible values.
///
/// Typically the `Hash` is delegated to the type contained within the `DamlValue` (i.e. `String` in the case of
/// `DamlValue::Text`) however for other variants such as `DamlValue::Map` and `DamlValue::GenMap` a custom
/// implementation is needed which in turn requires that a custom implementation of `PartialOrd` and `Ord` be defined
/// as well.
// As we ensure the invariant `k1 == k2 ⇒ hash(k1) == hash(k2)` holds we can safely suppress this lint.
#[allow(clippy::derive_hash_xor_eq)]
impl Hash for DamlValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DamlValue::Record(v) => v.hash(state),
            DamlValue::Variant(v) => v.hash(state),
            DamlValue::Enum(v) => v.hash(state),
            DamlValue::List(v) => v.hash(state),
            DamlValue::Int64(v) => v.hash(state),
            DamlValue::Numeric(v) => v.hash(state),
            DamlValue::Text(v) => v.hash(state),
            DamlValue::Party(v) => v.party.hash(state),
            DamlValue::ContractId(v) => v.contract_id.hash(state),
            DamlValue::Timestamp(v) => v.hash(state),
            DamlValue::Bool(v) => v.hash(state),
            DamlValue::Unit => {},
            DamlValue::Date(v) => v.hash(state),
            DamlValue::Optional(v) => v.hash(state),
            DamlValue::Map(v) => {
                let mut map_entries = v.iter().collect::<Vec<_>>();
                map_entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
                map_entries.iter().for_each(|kv| kv.hash(state));
            },
            DamlValue::GenMap(v) => {
                let mut map_entries = v.iter().collect::<Vec<_>>();
                map_entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
                map_entries.iter().for_each(|kv| kv.hash(state));
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
            (DamlValue::Map(v1), DamlValue::Map(v2)) => v1.keys().partial_cmp(v2.keys()),
            (DamlValue::GenMap(v1), DamlValue::GenMap(v2)) => v1.keys().partial_cmp(v2.keys()),
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
    use super::{DamlValue, Hash, HashMap, Hasher};
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_hash_and_eq_for_text() {
        let value1 = DamlValue::Text(String::from("text"));
        let value2 = DamlValue::Text(String::from("text"));
        let mut hasher1 = DefaultHasher::new();
        value1.hash(&mut hasher1);
        let hash1 = hasher1.finish();
        let mut hasher2 = DefaultHasher::new();
        value2.hash(&mut hasher2);
        let hash2 = hasher2.finish();
        assert_eq!(value1, value2);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_and_eq_for_i64() {
        let value1 = DamlValue::Int64(101);
        let value2 = DamlValue::Int64(101);
        let mut hasher1 = DefaultHasher::new();
        value1.hash(&mut hasher1);
        let hash1 = hasher1.finish();
        let mut hasher2 = DefaultHasher::new();
        value2.hash(&mut hasher2);
        let hash2 = hasher2.finish();
        assert_eq!(value1, value2);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_and_eq_for_map() {
        let items =
            vec![(String::from("text1"), DamlValue::Int64(100)), (String::from("text2"), DamlValue::Int64(200))];
        let value1 = DamlValue::Map(items.clone().into_iter().collect::<HashMap<String, DamlValue>>());
        let value2 = DamlValue::Map(items.into_iter().collect::<HashMap<String, DamlValue>>());
        let mut hasher1 = DefaultHasher::new();
        value1.hash(&mut hasher1);
        let hash1 = hasher1.finish();
        let mut hasher2 = DefaultHasher::new();
        value2.hash(&mut hasher2);
        let hash2 = hasher2.finish();
        assert_eq!(value1, value2);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_and_eq_for_genmap() {
        let items = vec![
            (DamlValue::Text(String::from("text1")), DamlValue::Int64(100)),
            (DamlValue::Text(String::from("text2")), DamlValue::Int64(200)),
        ];
        let value1 = DamlValue::GenMap(items.clone().into_iter().collect::<HashMap<_, _>>());
        let value2 = DamlValue::GenMap(items.into_iter().collect::<HashMap<_, _>>());
        let mut hasher1 = DefaultHasher::new();
        value1.hash(&mut hasher1);
        let hash1 = hasher1.finish();
        let mut hasher2 = DefaultHasher::new();
        value2.hash(&mut hasher2);
        let hash2 = hasher2.finish();
        assert_eq!(value1, value2);
        assert_eq!(hash1, hash2);
    }
}
