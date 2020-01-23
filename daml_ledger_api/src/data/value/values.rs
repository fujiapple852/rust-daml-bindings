use crate::data::value::{DamlEnum, DamlRecord, DamlVariant};
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::map::Entry;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::value::Sum;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::{Enum, List, Map, Optional, Record, Value, Variant};
use crate::util;
use crate::util::Required;
use bigdecimal::BigDecimal;
use chrono::Date;
use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use std::str::FromStr;

/// A generic representation of data on a DAML ledger.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DamlValue {
    Record(DamlRecord),
    Variant(DamlVariant),
    Enum(DamlEnum),
    ContractId(String),
    List(Vec<DamlValue>),
    Int64(i64),
    Numeric(BigDecimal),
    Text(String),
    Timestamp(DateTime<Utc>),
    Party(String),
    Bool(bool),
    Unit,
    Date(Date<Utc>),
    Optional(Option<Box<DamlValue>>),
    Map(HashMap<String, DamlValue>),
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

    pub fn new_contract_id(contract_id: impl Into<String>) -> Self {
        DamlValue::ContractId(contract_id.into())
    }

    pub fn new_list(list: impl Into<Vec<Self>>) -> Self {
        DamlValue::List(list.into())
    }

    pub fn new_int64(value: impl Into<i64>) -> Self {
        DamlValue::Int64(value.into())
    }

    pub fn new_numeric(numeric: impl Into<BigDecimal>) -> Self {
        DamlValue::Numeric(numeric.into())
    }

    pub fn new_text(text: impl Into<String>) -> Self {
        DamlValue::Text(text.into())
    }

    pub fn new_timestamp(timestamp: impl Into<DateTime<Utc>>) -> Self {
        DamlValue::Timestamp(timestamp.into())
    }

    pub fn new_party(party: impl Into<String>) -> Self {
        DamlValue::Party(party.into())
    }

    pub fn new_bool(value: bool) -> Self {
        DamlValue::Bool(value)
    }

    pub fn new_unit() -> Self {
        DamlValue::Unit
    }

    pub fn new_date(date: impl Into<Date<Utc>>) -> Self {
        DamlValue::Date(date.into())
    }

    pub fn new_optional(optional: Option<Self>) -> Self {
        DamlValue::Optional(optional.map(Box::new))
    }

    pub fn new_map(map: impl Into<HashMap<String, Self>>) -> Self {
        DamlValue::Map(map.into())
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

    pub fn try_date(&self) -> DamlResult<Date<Utc>> {
        match *self {
            DamlValue::Date(d) => Ok(d),
            _ => Err(self.make_unexpected_type_error("Date")),
        }
    }

    pub fn try_date_ref(&self) -> DamlResult<&Date<Utc>> {
        match self {
            DamlValue::Date(d) => Ok(d),
            _ => Err(self.make_unexpected_type_error("Date")),
        }
    }

    pub fn try_int64(&self) -> DamlResult<i64> {
        match self {
            DamlValue::Int64(i) => Ok(*i),
            _ => Err(self.make_unexpected_type_error("Int64")),
        }
    }

    pub fn try_int64_ref(&self) -> DamlResult<&i64> {
        match self {
            DamlValue::Int64(i) => Ok(i),
            _ => Err(self.make_unexpected_type_error("Int64")),
        }
    }

    pub fn try_numeric(&self) -> DamlResult<&BigDecimal> {
        match self {
            DamlValue::Numeric(d) => Ok(d),
            _ => Err(self.make_unexpected_type_error("Numeric")),
        }
    }

    // BigDecimal does not implement the Copy trait
    pub fn try_numeric_clone(&self) -> DamlResult<BigDecimal> {
        match self {
            DamlValue::Numeric(d) => Ok(d.clone()),
            _ => Err(self.make_unexpected_type_error("Numeric")),
        }
    }

    pub fn try_bool(&self) -> DamlResult<bool> {
        match self {
            DamlValue::Bool(b) => Ok(*b),
            _ => Err(self.make_unexpected_type_error("Bool")),
        }
    }

    pub fn try_bool_ref(&self) -> DamlResult<&bool> {
        match self {
            DamlValue::Bool(b) => Ok(b),
            _ => Err(self.make_unexpected_type_error("Bool")),
        }
    }

    pub fn try_text(&self) -> DamlResult<&str> {
        match self {
            DamlValue::Text(s) => Ok(&s[..]),
            _ => Err(self.make_unexpected_type_error("Text")),
        }
    }

    pub fn try_timestamp(&self) -> DamlResult<DateTime<Utc>> {
        match *self {
            DamlValue::Timestamp(ts) => Ok(ts),
            _ => Err(self.make_unexpected_type_error("Timestamp")),
        }
    }

    pub fn try_timestamp_ref(&self) -> DamlResult<&DateTime<Utc>> {
        match self {
            DamlValue::Timestamp(ts) => Ok(ts),
            _ => Err(self.make_unexpected_type_error("Timestamp")),
        }
    }

    pub fn try_party(&self) -> DamlResult<&str> {
        match self {
            DamlValue::Party(s) => Ok(&s[..]),
            _ => Err(self.make_unexpected_type_error("Party")),
        }
    }

    pub fn try_contract_id(&self) -> DamlResult<&str> {
        match self {
            DamlValue::ContractId(s) => Ok(&s[..]),
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
            DamlValue::Optional(opt) => Ok(opt.as_ref().map(AsRef::as_ref)),
            _ => Err(self.make_unexpected_type_error("Optional")),
        }
    }

    pub fn try_map(&self) -> DamlResult<&HashMap<String, Self>> {
        match self {
            DamlValue::Map(m) => Ok(m),
            _ => Err(self.make_unexpected_type_error("Map")),
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
    /// This method is designed to be used with the `daml_path!` macro from the `daml_ledger_macro` crate which
    /// provides a concise DSL for constructing a DAML data extractor closure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use daml_ledger_api::data::value::{DamlRecord, DamlValue, DamlRecordField};
    /// # use daml_ledger_api::data::{DamlResult, DamlError};
    /// # use daml_ledger_api::data::DamlIdentifier;
    /// # fn main() -> DamlResult<()> {
    /// let fields: Vec<DamlRecordField> = vec![DamlRecordField::new(Some("party"), DamlValue::new_party("Alice"))];
    /// let record: DamlRecord = DamlRecord::new(fields, None::<DamlIdentifier>);
    /// let record_value: DamlValue = DamlValue::new_record(record);
    /// let text_value: DamlValue = DamlValue::new_text("test");
    ///
    /// fn my_party_extractor(rec: &DamlRecord) -> DamlResult<&str> {
    ///     rec.field("party")?.try_party()
    /// }
    ///
    /// fn my_text_extractor(rec: &DamlRecord) -> DamlResult<&str> {
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
        F: Fn(&'a DamlRecord) -> DamlResult<R>,
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

impl TryFrom<Value> for DamlValue {
    type Error = DamlError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.sum {
            Some(sum) => {
                let convert = |sum| {
                    Ok(match sum {
                        Sum::Record(v) => DamlValue::Record(v.try_into()?),
                        Sum::Variant(v) => DamlValue::Variant((*v).try_into()?),
                        Sum::Enum(e) => DamlValue::Enum(e.into()),
                        Sum::ContractId(v) => DamlValue::ContractId(v),
                        Sum::List(v) => DamlValue::List(
                            v.elements.into_iter().map(TryInto::try_into).collect::<DamlResult<Vec<_>>>()?,
                        ),
                        Sum::Int64(v) => DamlValue::Int64(v),
                        Sum::Numeric(v) => DamlValue::Numeric(BigDecimal::from_str(&v)?),
                        Sum::Text(v) => DamlValue::Text(v),
                        Sum::Timestamp(v) => DamlValue::Timestamp(util::datetime_from_micros(v)?),
                        Sum::Party(v) => DamlValue::Party(v),
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
                    })
                };
                convert(sum)
            },
            None => Err(DamlError::new_failed_conversion("GRPC Value was None")),
        }
    }
}

impl From<DamlValue> for Value {
    fn from(daml_value: DamlValue) -> Self {
        Self {
            sum: match daml_value {
                DamlValue::Record(v) => Some(Sum::Record(Record::from(v))),
                DamlValue::Variant(v) => Some(Sum::Variant(Box::new(Variant::from(v)))),
                DamlValue::Enum(e) => Some(Sum::Enum(Enum::from(e))),
                DamlValue::ContractId(v) => Some(Sum::ContractId(v)),
                DamlValue::List(v) => Some(Sum::List(List {
                    elements: v.into_iter().map(Value::from).collect(),
                })),
                DamlValue::Int64(v) => Some(Sum::Int64(v)),
                // TODO: review the soundness of the numeric formatting here and consider using the `rust-decimal` crate
                DamlValue::Numeric(v) => Some(Sum::Numeric(format!("{:.37}", v))),
                DamlValue::Text(v) => Some(Sum::Text(v)), // value.set_text(v),
                DamlValue::Timestamp(v) => Some(Sum::Timestamp(v.timestamp())),
                DamlValue::Party(v) => Some(Sum::Party(v)),
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
                        .map(|(key, val)| Entry {
                            key,
                            value: Some(val.into()),
                        })
                        .collect(),
                })),
            },
        }
    }
}
