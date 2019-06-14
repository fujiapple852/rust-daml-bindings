use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::Date;
use chrono::DateTime;
use chrono::Utc;
use protobuf::well_known_types::Empty;
use protobuf::RepeatedField;

use crate::data::value::record::DamlRecord;
use crate::data::value::variant::DamlVariant;
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf_autogen::value::Optional;
use crate::grpc_protobuf_autogen::value::Value;
use crate::grpc_protobuf_autogen::value::Value_oneof_Sum;
use crate::grpc_protobuf_autogen::value::{List, Map, Map_Entry};
use crate::util;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

/// A generic representation of data on a DAML ledger.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DamlValue {
    Record(DamlRecord),
    Variant(DamlVariant),
    ContractId(String),
    List(Vec<DamlValue>),
    Int64(i64),
    Decimal(BigDecimal),
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

    pub fn new_contract_id(contract_id: impl Into<String>) -> Self {
        DamlValue::ContractId(contract_id.into())
    }

    pub fn new_list(list: impl Into<Vec<Self>>) -> Self {
        DamlValue::List(list.into())
    }

    pub fn new_int64(value: impl Into<i64>) -> Self {
        DamlValue::Int64(value.into())
    }

    pub fn new_decimal(decimal: impl Into<BigDecimal>) -> Self {
        DamlValue::Decimal(decimal.into())
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
        DamlValue::Optional(optional.and_then(|val| Some(Box::new(val))))
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

    pub fn try_decimal(&self) -> DamlResult<&BigDecimal> {
        match self {
            DamlValue::Decimal(d) => Ok(d),
            _ => Err(self.make_unexpected_type_error("Decimal")),
        }
    }

    // BigDecimal does not implement the Copy trait
    pub fn try_decimal_clone(&self) -> DamlResult<BigDecimal> {
        match self {
            DamlValue::Decimal(d) => Ok(d.clone()),
            _ => Err(self.make_unexpected_type_error("Decimal")),
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
            DamlValue::ContractId(_) => "ContractId",
            DamlValue::List(_) => "List",
            DamlValue::Int64(_) => "Int64",
            DamlValue::Decimal(_) => "Decimal",
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
        Self::new_decimal(d)
    }
}
impl From<f64> for DamlValue {
    fn from(d: f64) -> Self {
        Self::new_decimal(d)
    }
}

impl TryFrom<Value> for DamlValue {
    type Error = DamlError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.Sum {
            Some(sum) => {
                let convert = |sum| {
                    Ok(match sum {
                        Value_oneof_Sum::record(v) => DamlValue::Record(v.try_into()?),
                        Value_oneof_Sum::variant(v) => DamlValue::Variant(v.try_into()?),
                        Value_oneof_Sum::contract_id(v) => DamlValue::ContractId(v),
                        Value_oneof_Sum::list(mut v) => DamlValue::List(
                            (v.take_elements() as RepeatedField<Value>)
                                .into_iter()
                                .map(TryInto::try_into)
                                .collect::<DamlResult<Vec<_>>>()?,
                        ),
                        Value_oneof_Sum::int64(v) => DamlValue::Int64(v),
                        Value_oneof_Sum::decimal(v) => DamlValue::Decimal(BigDecimal::from_str(&v)?),
                        Value_oneof_Sum::text(v) => DamlValue::Text(v),
                        Value_oneof_Sum::timestamp(v) => DamlValue::Timestamp(util::datetime_from_micros(v)?),
                        Value_oneof_Sum::party(v) => DamlValue::Party(v),
                        Value_oneof_Sum::bool(v) => DamlValue::Bool(v),
                        Value_oneof_Sum::unit(_) => DamlValue::Unit,
                        Value_oneof_Sum::date(v) => DamlValue::Date(util::date_from_days(v)?),
                        Value_oneof_Sum::optional(mut v) => DamlValue::Optional(if v.has_value() {
                            Some(Box::new(v.take_value().try_into()?))
                        } else {
                            None
                        }),
                        Value_oneof_Sum::map(mut v) => DamlValue::Map(
                            (v.take_entries() as RepeatedField<Map_Entry>)
                                .into_iter()
                                .map(|mut v| Ok((v.take_key(), v.take_value().try_into()?)))
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
        let mut value = Self::new();
        match daml_value {
            DamlValue::Record(v) => value.set_record(v.into()),
            DamlValue::Variant(v) => value.set_variant(v.into()),
            DamlValue::ContractId(v) => value.set_contract_id(v),
            DamlValue::List(v) => {
                let mut list = List::new();
                list.set_elements(v.into_iter().map(Into::into).collect());
                value.set_list(list);
            },
            DamlValue::Int64(v) => value.set_int64(v),

            // TODO: review the soundness of the decimal formatting here and consider using the `rust-decimal` crate
            DamlValue::Decimal(v) => value.set_decimal(format!("{:.10}", v)),
            DamlValue::Text(v) => value.set_text(v),
            DamlValue::Timestamp(v) => value.set_timestamp(v.timestamp()),
            DamlValue::Party(v) => value.set_party(v),
            DamlValue::Bool(v) => value.set_bool(v),
            DamlValue::Unit => value.set_unit(Empty::new()),
            DamlValue::Date(v) => value.set_date(util::days_from_date(v)),
            DamlValue::Optional(Some(v)) => {
                let mut opt = Optional::new();
                opt.set_value((*v).into());
                value.set_optional(opt);
            },
            DamlValue::Optional(None) => {
                value.set_optional(Optional::new());
            },
            DamlValue::Map(v) => {
                let mut map = Map::new();
                map.set_entries(
                    v.into_iter()
                        .map(|(key, val)| {
                            let mut entry = Map_Entry::new();
                            entry.set_key(key);
                            entry.set_value(val.into());
                            entry
                        })
                        .collect(),
                );
                value.set_map(map);
            },
        };
        value
    }
}
