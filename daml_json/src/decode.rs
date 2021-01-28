use crate::error::{DamlJsonCodecError, DamlJsonCodecResult};
use crate::util::{AsSingleSliceExt, Required};
use chrono::{offset, Date, DateTime, NaiveDate};
use daml_grpc::data::value::{DamlEnum, DamlRecord, DamlRecordField, DamlValue, DamlVariant};
use daml_grpc::data::DamlIdentifier;
use daml_grpc::primitive_types::{DamlGenMap, DamlInt64, DamlNumeric, DamlTextMap};
use daml_lf::element;
use daml_lf::element::{DamlArchive, DamlData, DamlField, DamlType};
use serde_json::{Map, Value};
use std::convert::TryFrom;
use std::str::FromStr;

/// Decode a `DamlValue` from JSON.
#[derive(Debug)]
pub struct JsonDecoder<'a> {
    arc: &'a DamlArchive<'a>,
}

impl<'a> JsonDecoder<'a> {
    pub const fn new(arc: &'a DamlArchive<'a>) -> Self {
        Self {
            arc,
        }
    }

    /// Decode a `DamlValue` from a JSON `Value` for a given `DamlType`.
    ///
    /// See [`JsonEncoder`] and the
    /// (DAML LF JSON Encoding specification)[https://docs.daml.com/json-api/lf-value-specification.html] for details.
    ///
    /// [`JsonEncoder`]: crate::encode::JsonEncoder
    pub fn decode(&self, json: &Value, ty: &DamlType<'_>) -> DamlJsonCodecResult<DamlValue> {
        self.do_decode(json, ty, true)
    }

    /// Perform the decode.
    ///
    /// Here `top_level` refers to whether we are processing a value corresponding to the "top level" of a type or a
    /// nested types.  This is required to support the "shortcut" decoding for optional fields.
    fn do_decode(&self, json: &Value, ty: &DamlType<'_>, top_level: bool) -> DamlJsonCodecResult<DamlValue> {
        match ty {
            DamlType::Unit =>
                if json.try_object()?.is_empty() {
                    Ok(DamlValue::Unit)
                } else {
                    Err(DamlJsonCodecError::UnexpectedUnitData)
                },
            DamlType::Bool => Self::decode_bool(json),
            DamlType::Int64 => Self::decode_int64(json),
            DamlType::Text => Self::decode_text(json),
            DamlType::Party => Self::decode_party(json),
            DamlType::ContractId(_) => Self::decode_contract_id(json),
            DamlType::Numeric(_) => Self::decode_numeric(json),
            DamlType::Date => Self::decode_date(json),
            DamlType::Timestamp => Self::decode_timestamp(json),
            DamlType::List(tys) => Ok(DamlValue::List(
                json.try_array()?
                    .iter()
                    .map(|item| self.do_decode(item, tys.as_single()?, true))
                    .collect::<DamlJsonCodecResult<Vec<_>>>()?,
            )),
            DamlType::TextMap(tys) => Ok(DamlValue::Map(
                json.try_object()?
                    .iter()
                    .map(|(k, v)| Ok((k.clone(), self.do_decode(v, tys.as_single()?, true)?)))
                    .collect::<DamlJsonCodecResult<DamlTextMap<DamlValue>>>()?,
            )),
            DamlType::GenMap(tys) => {
                let array = json.try_array()?;
                let genmap = array
                    .iter()
                    .map(|item| match item.try_array()?.as_slice() {
                        [k, v] => Ok((
                            self.do_decode(k, tys.first().req()?, true)?,
                            self.do_decode(v, tys.last().req()?, true)?,
                        )),
                        _ => Err(DamlJsonCodecError::UnexpectedGenMapTypes),
                    })
                    .collect::<DamlJsonCodecResult<DamlGenMap<DamlValue, DamlValue>>>()?;

                // If the resulting GenMap containers fewer entries that the input array then we know that the input
                // array must have contained duplicate keys and should therefore be rejected.
                if array.len() == genmap.len() {
                    Ok(DamlValue::GenMap(genmap))
                } else {
                    Err(DamlJsonCodecError::DuplicateGenMapKeys)
                }
            },
            DamlType::TyCon(tycon) | DamlType::BoxedTyCon(tycon) => self.decode_data(
                json,
                self.arc
                    .data_by_tycon(tycon)
                    .ok_or_else(|| DamlJsonCodecError::DataNotFound(tycon.tycon().to_string()))?,
            ),
            DamlType::Optional(nested) => {
                let single = nested.as_single()?;
                if top_level {
                    if json.is_null() {
                        Ok(DamlValue::Optional(None))
                    } else {
                        Ok(DamlValue::Optional(Some(Box::new(self.do_decode(json, single, false)?))))
                    }
                } else {
                    match json.try_array()?.as_slice() {
                        [] => Ok(DamlValue::Optional(None)),
                        [inner_json] =>
                            Ok(DamlValue::Optional(Some(Box::new(self.do_decode(inner_json, single, false)?)))),
                        _ => Err(DamlJsonCodecError::UnexpectedOptionalArrayLength),
                    }
                }
            },
            DamlType::Var(_)
            | DamlType::Nat(_)
            | DamlType::Arrow
            | DamlType::Any
            | DamlType::TypeRep
            | DamlType::Update
            | DamlType::Scenario
            | DamlType::Forall(_)
            | DamlType::Struct(_)
            | DamlType::Syn(_) => Err(DamlJsonCodecError::UnsupportedDamlType(ty.name().to_owned())),
        }
    }

    fn decode_bool(json: &Value) -> DamlJsonCodecResult<DamlValue> {
        Ok(DamlValue::Bool(json.try_bool()?))
    }

    fn decode_int64(json: &Value) -> DamlJsonCodecResult<DamlValue> {
        match (json.as_i64(), json.as_str()) {
            (Some(i64), None) => Ok(DamlValue::new_int64(i64)),
            (None, Some(s)) => Ok(DamlValue::new_int64(DamlInt64::from_str(s)?)),
            _ => Err(DamlJsonCodecError::UnexpectedJsonType(
                "i64 or String".to_owned(),
                Value::json_value_name(json).to_owned(),
            )),
        }
    }

    fn decode_numeric(json: &Value) -> DamlJsonCodecResult<DamlValue> {
        match (json.as_f64(), json.as_str()) {
            (Some(f64), None) => Ok(DamlValue::new_numeric(DamlNumeric::try_from(f64)?)),
            (None, Some(s)) => Ok(DamlValue::new_numeric(DamlNumeric::from_str(s)?)),
            _ => Err(DamlJsonCodecError::UnexpectedJsonType(
                "f64 or String".to_owned(),
                Value::json_value_name(json).to_owned(),
            )),
        }
    }

    fn decode_date(json: &Value) -> DamlJsonCodecResult<DamlValue> {
        Ok(DamlValue::new_date(Date::from_utc(NaiveDate::parse_from_str(json.try_string()?, "%Y-%m-%d")?, offset::Utc)))
    }

    fn decode_timestamp(json: &Value) -> DamlJsonCodecResult<DamlValue> {
        Ok(DamlValue::new_timestamp(DateTime::parse_from_rfc3339(json.try_string()?)?))
    }

    fn decode_text(json: &Value) -> DamlJsonCodecResult<DamlValue> {
        Ok(DamlValue::new_text(json.try_string()?))
    }

    fn decode_party(json: &Value) -> DamlJsonCodecResult<DamlValue> {
        Ok(DamlValue::new_party(json.try_string()?))
    }

    fn decode_contract_id(json: &Value) -> DamlJsonCodecResult<DamlValue> {
        Ok(DamlValue::new_contract_id(json.try_string()?))
    }

    /// Decode a `DamlValue` from a JSON `Value` and `DamlData`.
    fn decode_data(&self, json: &Value, data: &DamlData<'_>) -> DamlJsonCodecResult<DamlValue> {
        match data {
            DamlData::Template(template) => self.decode_record(json, template.fields()),
            DamlData::Record(record) => self.decode_record(json, record.fields()),
            DamlData::Variant(variant) => self.decode_variant(json, variant.fields()),
            DamlData::Enum(data_enum) => Self::decode_enum(json, data_enum),
        }
    }

    /// Decode a `DamlValue::Enum` from a JSON `Value` and `DamlEnum` fields.
    fn decode_enum(json: &Value, data_enum: &element::DamlEnum<'_>) -> DamlJsonCodecResult<DamlValue> {
        let constructor = json.try_string()?;
        if data_enum.constructors().any(|c| c == constructor) {
            Ok(DamlValue::Enum(DamlEnum::new(constructor, None::<DamlIdentifier>)))
        } else {
            Err(DamlJsonCodecError::UnknownEnumConstructor(constructor.to_owned()))
        }
    }

    /// Decode a `DamlValue::Variant` from a JSON `Value` and `DamlVariant` fields.
    fn decode_variant(&self, json: &Value, constructors: &[DamlField<'_>]) -> DamlJsonCodecResult<DamlValue> {
        let object = json.try_object()?;
        let tag = object.get("tag").req()?.try_string()?;
        let value = object.get("value").req()?;
        let constructor = constructors
            .iter()
            .find(|&field| field.name() == tag)
            .ok_or_else(|| DamlJsonCodecError::UnknownVariantConstructor(tag.to_owned()))?;
        Ok(DamlValue::Variant(DamlVariant::new(
            constructor.name(),
            Box::new(self.decode(value, constructor.ty())?),
            None::<DamlIdentifier>,
        )))
    }

    /// Decode a `DamlValue::Record` from a JSON `Value` and `DamlRecord` / `DamlTemplate` fields.
    fn decode_record(&self, json: &Value, lf_fields: &[DamlField<'_>]) -> DamlJsonCodecResult<DamlValue> {
        let fields = match (json.as_object(), json.as_array()) {
            (Some(obj), None) => self.decode_record_object(obj, lf_fields)?,
            (None, Some(arr)) => self.decode_record_array(arr, lf_fields)?,
            _ =>
                return Err(DamlJsonCodecError::UnexpectedJsonType(
                    "Object or Array".to_owned(),
                    Value::json_value_name(json).to_owned(),
                )),
        };
        Ok(DamlValue::Record(DamlRecord::new(fields, None::<DamlIdentifier>)))
    }

    fn decode_record_object(
        &self,
        obj: &Map<String, Value>,
        lf_fields: &[DamlField<'_>],
    ) -> DamlJsonCodecResult<Vec<DamlRecordField>> {
        lf_fields
            .iter()
            .map(|field| {
                let field_name = field.name();
                let field_ty = field.ty();
                let field_json = obj.get(field_name);
                match (field_ty, field_json) {
                    (DamlType::Optional(_), None) =>
                        Ok(DamlRecordField::new(Some(field_name), DamlValue::Optional(None))),
                    (_, Some(json)) => Ok(DamlRecordField::new(Some(field_name), self.decode(json, field_ty)?)),
                    (_, None) => Err(DamlJsonCodecError::MissingJsonRecordObjectField(field_name.to_owned())),
                }
            })
            .collect::<DamlJsonCodecResult<Vec<DamlRecordField>>>()
    }

    fn decode_record_array(
        &self,
        arr: &[Value],
        lf_fields: &[DamlField<'_>],
    ) -> DamlJsonCodecResult<Vec<DamlRecordField>> {
        lf_fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let json = arr
                    .get(i)
                    .ok_or_else(|| DamlJsonCodecError::MissingJsonRecordArrayField(i, field.name().to_owned()))?;
                Ok(DamlRecordField::new(Some(field.name()), self.decode(json, field.ty())?))
            })
            .collect::<DamlJsonCodecResult<Vec<DamlRecordField>>>()
    }
}

/// Extension trait for JSON `Value` to add `try_xxx` methods.
pub trait JsonTryAsExt {
    fn try_null(&self) -> DamlJsonCodecResult<()>;
    fn try_bool(&self) -> DamlJsonCodecResult<bool>;
    fn try_int64(&self) -> DamlJsonCodecResult<i64>;
    fn try_string(&self) -> DamlJsonCodecResult<&str>;
    fn try_array(&self) -> DamlJsonCodecResult<&Vec<Value>>;
    fn try_object(&self) -> DamlJsonCodecResult<&Map<String, Value>>;
    fn make_unexpected_type_error(value: &Value, expected: &str) -> DamlJsonCodecError {
        DamlJsonCodecError::UnexpectedJsonType(expected.to_owned(), Self::json_value_name(value).to_owned())
    }
    fn json_value_name(value: &Value) -> &str {
        match value {
            Value::Null => "Null",
            Value::Bool(_) => "Bool",
            Value::Number(_) => "Number",
            Value::String(_) => "String",
            Value::Array(_) => "Array",
            Value::Object(_) => "Object",
        }
    }
}

impl JsonTryAsExt for Value {
    fn try_null(&self) -> DamlJsonCodecResult<()> {
        self.as_null().ok_or_else(|| Self::make_unexpected_type_error(self, "Null"))
    }

    fn try_bool(&self) -> DamlJsonCodecResult<bool> {
        self.as_bool().ok_or_else(|| Self::make_unexpected_type_error(self, "Bool"))
    }

    fn try_int64(&self) -> DamlJsonCodecResult<i64> {
        self.as_i64().ok_or_else(|| Self::make_unexpected_type_error(self, "Number(i64)"))
    }

    fn try_string(&self) -> DamlJsonCodecResult<&str> {
        self.as_str().ok_or_else(|| Self::make_unexpected_type_error(self, "String"))
    }

    fn try_array(&self) -> DamlJsonCodecResult<&Vec<Value>> {
        self.as_array().ok_or_else(|| Self::make_unexpected_type_error(self, "Array"))
    }

    fn try_object(&self) -> DamlJsonCodecResult<&Map<String, Value>> {
        self.as_object().ok_or_else(|| Self::make_unexpected_type_error(self, "Object"))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DamlArchive, DamlEnum, DamlJsonCodecError, DamlJsonCodecResult, DamlType, DamlValue, JsonDecoder, Value,
    };
    use daml::macros::daml_value;
    use daml_grpc::primitive_types::DamlTextMap;
    use daml_lf::element::{DamlAbsoluteTyCon, DamlTyCon, DamlTyConName};
    use daml_lf::DarFile;
    use maplit::hashmap;
    use serde_json::json;
    use std::borrow::Cow;

    static TESTING_TYPES_DAR_PATH: &str =
        "../resources/testing_types_sandbox/archive/TestingTypes-1_0_0-sdk_1_9_0-lf_1_11.dar";

    /// `{}` -> `() : ()`
    #[test]
    fn test_unit() -> DamlJsonCodecResult<()> {
        let json_value = json!({});
        let ty = DamlType::Unit;
        let expected = daml_value!();
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `true` -> `True : bool`
    #[test]
    fn test_bool() -> DamlJsonCodecResult<()> {
        let json_value = json!(true);
        let ty = DamlType::Bool;
        let expected = daml_value!(true);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `42` -> `42 : Int`
    #[test]
    fn test_int64() -> DamlJsonCodecResult<()> {
        let json_value = json!(42);
        let ty = DamlType::Int64;
        let expected = daml_value!(42);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `-42` -> `-42 : Int`
    #[test]
    fn test_int64_neg() -> DamlJsonCodecResult<()> {
        let json_value = json!(-42);
        let ty = DamlType::Int64;
        let expected = daml_value!(-42);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"42"` -> `42 : Int`
    #[test]
    fn test_int64_string() -> DamlJsonCodecResult<()> {
        let json_value = json!("42");
        let ty = DamlType::Int64;
        let expected = daml_value!(42);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"-42"` -> `-42 : Int`
    #[test]
    fn test_int64_neg_string() -> DamlJsonCodecResult<()> {
        let json_value = json!("-42");
        let ty = DamlType::Int64;
        let expected = daml_value!(-42);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `4.2` -> `4.2 : Int`
    #[test]
    fn test_int64_fails() -> DamlJsonCodecResult<()> {
        let json_value = json!(4.2);
        let ty = DamlType::Int64;
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty);
        assert!(actual.is_err());
        Ok(())
    }

    /// `1.0` -> `1.0 : Decimal`
    #[test]
    fn test_numeric_f64() -> DamlJsonCodecResult<()> {
        let json_value = json!(1.0);
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(10)));
        let expected = daml_value!(1.0);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `-1.0` -> `-1.0 : Decimal`
    #[test]
    fn test_numeric_f64_neg() -> DamlJsonCodecResult<()> {
        let json_value = json!(-1.0);
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(10)));
        let expected = daml_value!(-1.0);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"1.23"` -> `1.23 : Decimal`
    #[test]
    fn test_numeric_string() -> DamlJsonCodecResult<()> {
        let json_value = json!("1.23");
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(10)));
        let expected = daml_value!(1.23);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"-1.23"` -> `-1.23 : Decimal`
    #[test]
    fn test_numeric_string_neg() -> DamlJsonCodecResult<()> {
        let json_value = json!("-1.23");
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(10)));
        let expected = daml_value!(-1.23);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `0.30000000000000004` -> `0.3 : Decimal`
    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_numeric_f64_round() -> DamlJsonCodecResult<()> {
        let json_value = json!(0.30000000000000004);
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(10)));
        let expected = daml_value!(0.3);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `2e3` -> `2000 : Decimal`
    #[test]
    fn test_numeric_f64_sci() -> DamlJsonCodecResult<()> {
        let json_value = json!(2e3);
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(10)));
        let expected = daml_value!(2000.0);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `-0` -> `0 : Decimal`
    #[test]
    fn test_numeric_f64_neg_zero() -> DamlJsonCodecResult<()> {
        let json_value = json!(-0);
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(10)));
        let expected = daml_value!(0.0);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `9999999999999999999999999999.9999999999` -> `9999999999999999999999999999.9999999999 : Decimal`
    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_numeric_f64_large() -> DamlJsonCodecResult<()> {
        let json_value = json!(9999999999999999999999999999.9999999999);
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(10)));
        let expected = daml_value!(9999999999999999999999999999.9999999999);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `42` -> `42 : Decimal`
    #[test]
    fn test_numeric_f64_whole() -> DamlJsonCodecResult<()> {
        let json_value = json!(42);
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(10)));
        let expected = daml_value!(42.0);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `42` -> `42 : Decimal`
    #[test]
    fn test_numeric_string_whole() -> DamlJsonCodecResult<()> {
        let json_value = json!("42");
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(10)));
        let expected = daml_value!(42.0);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"blah"` -> `n/a : Decimal`
    #[test]
    fn test_numeric_string_fails_garbage() -> DamlJsonCodecResult<()> {
        let json_value = json!("blah");
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(10)));
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty);
        assert!(actual.is_err());
        Ok(())
    }

    /// `"  42  "` -> `n/a : Decimal`
    #[test]
    fn test_numeric_string_fails_whitespace() -> DamlJsonCodecResult<()> {
        let json_value = json!("  42  ");
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(10)));
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty);
        assert!(actual.is_err());
        Ok(())
    }

    /// `"test"` -> `"test" : Text`
    #[test]
    fn test_text() -> DamlJsonCodecResult<()> {
        let json_value = json!("test");
        let ty = DamlType::Text;
        let expected = daml_value!("test");
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `""` -> `"" : Text`
    #[test]
    fn test_text_empty() -> DamlJsonCodecResult<()> {
        let json_value = json!("");
        let ty = DamlType::Text;
        let expected = daml_value!("");
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"2019-06-18"` -> `date 2019 Jun 18 : Text`
    #[test]
    fn test_date() -> DamlJsonCodecResult<()> {
        let json_value = json!("2019-06-18");
        let ty = DamlType::Date;
        let expected = daml_value!("2019-06-18"#d);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"9999-12-31"` -> `date 9999 Dec 31 : Text`
    #[test]
    fn test_date_min() -> DamlJsonCodecResult<()> {
        let json_value = json!("9999-12-31");
        let ty = DamlType::Date;
        let expected = daml_value!("9999-12-31"#d);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"0001-01-01"` -> `date 0001 Jan 01 : Text`
    #[test]
    fn test_date_max() -> DamlJsonCodecResult<()> {
        let json_value = json!("0001-01-01");
        let ty = DamlType::Date;
        let expected = daml_value!("0001-01-01"#d);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"9999-99-99"` -> `n/a : Text`
    #[test]
    fn test_date_invalid_fails() -> DamlJsonCodecResult<()> {
        let json_value = json!("9999-99-99");
        let ty = DamlType::Date;
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty);
        assert!(actual.is_err());
        Ok(())
    }

    /// `"1990-11-09T04:30:23.1234569Z"` -> `datetime 1990 Nov 09 04 30 23 xxx : Text`
    #[test]
    fn test_timestamp_full() -> DamlJsonCodecResult<()> {
        let json_value = json!("1990-11-09T04:30:23.1234569Z");
        let ty = DamlType::Timestamp;
        let expected = daml_value!("1990-11-09T04:30:23.1234569Z"#t);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"1990-11-09T04:30:23Z"` -> `datetime 1990 Nov 09 04 30 23 xxx : Text`
    #[test]
    fn test_timestamp_no_sub_sec() -> DamlJsonCodecResult<()> {
        let json_value = json!("1990-11-09T04:30:23Z");
        let ty = DamlType::Timestamp;
        let expected = daml_value!("1990-11-09T04:30:23Z"#t);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"1990-11-09T04:30:23.123Z"` -> `datetime 1990 Nov 09 04 30 23 xxx : Text`
    #[test]
    fn test_timestamp_no_micros() -> DamlJsonCodecResult<()> {
        let json_value = json!("1990-11-09T04:30:23.123Z");
        let ty = DamlType::Timestamp;
        let expected = daml_value!("1990-11-09T04:30:23.123Z"#t);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"0001-01-01T00:00:00Z"` -> `datetime 0001 Jan 01 00 00 00 xxx : Text`
    #[test]
    fn test_timestamp_min() -> DamlJsonCodecResult<()> {
        let json_value = json!("0001-01-01T00:00:00Z");
        let ty = DamlType::Timestamp;
        let expected = daml_value!("0001-01-01T00:00:00Z"#t);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"9999-12-31T23:59:59.999999Z"` -> `datetime 9999 Dec 12 59 59 59 xxx : Text`
    #[test]
    fn test_timestamp_max() -> DamlJsonCodecResult<()> {
        let json_value = json!("9999-12-31T23:59:59.999999Z");
        let ty = DamlType::Timestamp;
        let expected = daml_value!("9999-12-31T23:59:59.999999Z"#t);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"Alice"` -> `"Alice" : Party`
    #[test]
    fn test_party() -> DamlJsonCodecResult<()> {
        let json_value = json!("Alice");
        let ty = DamlType::Party;
        let expected = daml_value!("Alice"#p);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `"foo:bar#baz"` -> `n/a : ContractId T`
    #[test]
    fn test_contract_id() -> DamlJsonCodecResult<()> {
        let json_value = json!("foo:bar#baz");
        let ty = DamlType::ContractId(None);
        let expected = daml_value!("foo:bar#baz"#c);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `null` -> `None : Optional Int64`
    #[test]
    fn test_opt_int_null() -> DamlJsonCodecResult<()> {
        let json_value = json!(null);
        let ty = DamlType::Optional(vec![DamlType::Int64]);
        let expected = daml_value!({?!});
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[]` -> `None : Optional Int64`
    #[test]
    fn test_opt_int_null_fails() -> DamlJsonCodecResult<()> {
        let json_value = json!([]);
        let ty = DamlType::Optional(vec![DamlType::Int64]);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty);
        assert!(actual.is_err());
        Ok(())
    }

    /// `[null]` -> `Some None : Optional (Optional Int64)`
    #[test]
    fn test_opt_opt_int_some_should_fail() -> DamlJsonCodecResult<()> {
        let json_value = json!([null]);
        let ty = DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Int64])]);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty);
        assert!(actual.is_err());
        Ok(())
    }

    /// `null` -> `None : Optional (Optional Int64)`
    #[test]
    fn test_opt_opt_int_null() -> DamlJsonCodecResult<()> {
        let json_value = json!(null);
        let ty = DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Int64])]);
        let expected = daml_value!({?!});
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `42` -> `Some 42 : Optional Int64`
    #[test]
    fn test_opt_int_some() -> DamlJsonCodecResult<()> {
        let json_value = json!(42);
        let ty = DamlType::Optional(vec![DamlType::Int64]);
        let expected = daml_value!({?=42});
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[]` -> `Some None : Optional (Optional Int64)`
    #[test]
    fn test_opt_opt_int_some_none() -> DamlJsonCodecResult<()> {
        let json_value = json!([]);
        let ty = DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Int64])]);
        let expected = daml_value!({?={?!}});
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[42]` -> `Some (Some 42) : Optional (Optional Int64)`
    #[test]
    fn test_opt_opt_int_some_some() -> DamlJsonCodecResult<()> {
        let json_value = json!([42]);
        let ty = DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Int64])]);
        let expected = daml_value!({?={?=42}});
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[[]]` -> `Some (Some None) : Optional (Optional (Optional Int64))`
    #[test]
    fn test_opt_opt_opt_int_some_some_none() -> DamlJsonCodecResult<()> {
        let json_value = json!([[]]);
        let ty = DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Int64])])]);
        let expected = daml_value!({?={?={?!}}});
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[[42]]` -> `Some (Some (Some 42)) : Optional (Optional (Optional Int64))`
    #[test]
    fn test_opt_opt_opt_int_some_some_some() -> DamlJsonCodecResult<()> {
        let json_value = json!([[42]]);
        let ty = DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Int64])])]);
        let expected = daml_value!({?={?={?=42}}});
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[[[]]]` -> `Some (Some (Some None)) : Optional (Optional (Optional (Optional Int64)))`
    #[test]
    fn test_opt_opt_opt_opt_int_some_some_some_none() -> DamlJsonCodecResult<()> {
        let json_value = json!([[[]]]);
        let ty =
            DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Optional(vec![
                DamlType::Int64,
            ])])])]);
        let expected = daml_value!({?={?={?={?!}}}});
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[[[42]]]` -> `Some (Some (Some 42)) : Optional (Optional (Optional (Optional Int64)))`
    #[test]
    fn test_opt_opt_opt_opt_int_some_some_some_some() -> DamlJsonCodecResult<()> {
        let json_value = json!([[[42]]]);
        let ty =
            DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Optional(vec![
                DamlType::Int64,
            ])])])]);
        let expected = daml_value!({?={?={?={?=42}}}});
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `null` -> `None : Optional Unit`
    #[test]
    fn test_opt_unit_null() -> DamlJsonCodecResult<()> {
        let json_value = json!(null);
        let ty = DamlType::Optional(vec![DamlType::Unit]);
        let expected = daml_value!({?!});
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{}` -> `None : Optional Unit`
    #[test]
    fn test_opt_unit_some() -> DamlJsonCodecResult<()> {
        let json_value = json!({});
        let ty = DamlType::Optional(vec![DamlType::Unit]);
        let expected = daml_value!({?=()});
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{ "landlord": "Alice", ... }` -> `RentalAgreement with landlord = "Alice"; ... : RentalAgreement`
    #[test]
    fn test_record() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({
             "landlord": "Alice",
             "tenant": "Bob",
             "terms": "test terms",
        });
        let ty = make_tycon("RentalAgreement", &dar.main.hash, vec!["DA", "RentDemo"]);
        let expected = daml_value!({landlord: "Alice"#p, tenant: "Bob"#p, terms: "test terms"});
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{ "landlord": "Alice", ... }` -> `RentalAgreement with landlord = "Alice"; ... : RentalAgreement`
    #[test]
    fn test_record_array() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!(["Alice", "Bob", 0]);
        let ty = make_tycon("Ping", &dar.main.hash, vec!["DA", "PingPong"]);
        let expected = daml_value!({sender: "Alice"#p, receiver: "Bob"#p, count: 0});
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{}` -> `Depth1 with foo = None : data Depth1 = Depth1 with foo: Optional Int64`
    #[test]
    fn test_record_depth1_omitted() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({});
        let ty = make_tycon("Depth1", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = daml_value!({foo: {?!}});
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{ "foo": null }` -> `Depth1 with foo = None : data Depth1 = Depth1 with foo: Optional Int64`
    #[test]
    fn test_record_depth1_none() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({ "foo": null });
        let ty = make_tycon("Depth1", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = daml_value!({foo: {?!}});
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{ "foo": 42 }` -> `Depth1 with foo = Some 42 : data Depth1 = Depth1 with foo: Optional Int64`
    #[test]
    fn test_record_depth1_some() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({ "foo": 42 });
        let ty = make_tycon("Depth1", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = daml_value!({foo: {?=42}});
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{}` -> `Depth2 with foo = None : data Depth2 = Depth2 with foo: Optional (Optional Int64)`
    #[test]
    fn test_record_depth2_omitted() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({});
        let ty = make_tycon("Depth2", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = daml_value!({foo: {?!}});
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{ "foo": null }` -> `Depth2 with foo = None : data Depth2 = Depth2 with foo: Optional (Optional Int64)`
    #[test]
    fn test_record_depth2_none() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({ "foo": null });
        let ty = make_tycon("Depth2", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = daml_value!({foo: {?!}});
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{ "foo": [] }` -> `Depth2 with foo = Some (None) : data Depth2 = Depth2 with foo: Optional (Optional Int64)`
    #[test]
    fn test_record_depth2_some_none() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({ "foo": [] });
        let ty = make_tycon("Depth2", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = daml_value!({foo: {?={?!}}});
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{"foo": [42]}` -> `Depth2 with foo = Some (Some 42) : data Depth2 = Depth2 with foo: Optional (Optional Int64)`
    #[test]
    fn test_record_depth2_some_some() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({ "foo": [42] });
        let ty = make_tycon("Depth2", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = daml_value!({foo: {?={?=42}}});
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[]` -> `[] : [Text]`
    #[test]
    fn test_list_bool_empty() -> DamlJsonCodecResult<()> {
        let json_value = json!([]);
        let ty = DamlType::List(vec![DamlType::Bool]);
        let expected = daml_value!([]);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `["a", "b", "c"]` -> `["a", "b", "c"] : [Text]`
    #[test]
    fn test_list_text() -> DamlJsonCodecResult<()> {
        let json_value = json!(["a", "b", "c"]);
        let ty = DamlType::List(vec![DamlType::Text]);
        let expected = daml_value!(["a", "b", "c"]);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `["a", null, "c"]` -> `[Some "a", None, Some "c"] : [Optional Text]`
    #[test]
    fn test_list_opt_text() -> DamlJsonCodecResult<()> {
        let json_value = json!(["a", null, "c"]);
        let ty = DamlType::List(vec![DamlType::Optional(vec![DamlType::Text])]);
        let expected = daml_value!([{?="a"}, {?!}, {?="c"}]);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[42, null, "c"]` -> n/a (error case)
    #[test]
    fn test_list_opt_mixed_fails() -> DamlJsonCodecResult<()> {
        let json_value = json!([42, null, "c"]);
        let ty = DamlType::List(vec![DamlType::Optional(vec![DamlType::Text])]);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty);
        assert!(actual.is_err());
        Ok(())
    }

    /// `[{...}]` -> `[RentalAgreement with landlord = "..."; tenant = "..."; terms = "..."] : [RentalAgreement]`
    #[test]
    fn test_list_record() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!([{
            "landlord": "Alice",
            "tenant": "Bob",
            "terms": "test terms",
        },
        {
            "landlord": "John",
            "tenant": "Paul",
            "terms": "more test terms",
        }]);
        let ty = DamlType::List(vec![make_tycon("RentalAgreement", &dar.main.hash, vec!["DA", "RentDemo"])]);
        let expected = daml_value!([{
            landlord: "Alice"#p,
            tenant: "Bob"#p,
            terms: "test terms"
        },
        {
            landlord: "John"#p,
            tenant: "Paul"#p,
            terms: "more test terms"
        }]);
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{}` -> `M.fromList [] : Map Int`
    #[test]
    fn test_textmap_int_empty() -> DamlJsonCodecResult<()> {
        let json_value = json!({});
        let ty = DamlType::TextMap(vec![DamlType::Int64]);
        let expected = DamlValue::Map(vec![].into_iter().collect());
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{"foo": 42, "bar": 43}` -> `M.fromList [("foo", 42), ("bar", 43)] : Map Int`
    #[test]
    fn test_textmap_int() -> DamlJsonCodecResult<()> {
        let json_value = json!({"foo": 42, "bar": 43});
        let ty = DamlType::TextMap(vec![DamlType::Int64]);
        let expected = DamlValue::Map(
            vec![("foo".to_owned(), daml_value![42]), ("bar".to_owned(), daml_value![43])].into_iter().collect(),
        );
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[{"foo": 42}, {"bar": 43}]` -> `[M.fromList [("foo", 42)], M.fromList [("bar", 43)]] : [Map Int]`
    #[test]
    fn test_list_textmap_int() -> DamlJsonCodecResult<()> {
        let json_value = json!([{"foo": 42}, {"bar": 43}]);
        let ty = DamlType::List(vec![DamlType::TextMap(vec![DamlType::Int64])]);
        let expected = daml_value![[
            (DamlValue::Map(DamlTextMap::from(hashmap! {"foo".to_owned() => daml_value![42]}))),
            (DamlValue::Map(DamlTextMap::from(hashmap! {"bar".to_owned() => daml_value![43]})))
        ]];
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{"foo": [1, 2, 3], "bar": [4, 5, 6]}` -> `M.fromList [("foo", [1, 2, 3]), ("bar", [4, 5, 6])] : Map [Int]`
    #[test]
    fn test_textmap_list_int() -> DamlJsonCodecResult<()> {
        let json_value = json!({"foo": [1, 2, 3], "bar": [4, 5, 6]});
        let ty = DamlType::TextMap(vec![DamlType::List(vec![DamlType::Int64])]);
        let expected = DamlValue::Map(DamlTextMap::from(hashmap! {
            "foo".to_owned() => daml_value![[1, 2, 3]],
            "bar".to_owned() => daml_value![[4, 5, 6]]
        }));
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{"first": {...}, last: {...}}` -> `M.fromList [("first", Agreement with ...)] : Map Agreement`
    #[test]
    fn test_textmap_record() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({
        "first": {
            "landlord": "Alice",
            "tenant": "Bob",
            "terms": "test terms",
        },
        "last": {
            "landlord": "John",
            "tenant": "Paul",
            "terms": "more test terms",
        }});
        let ty = DamlType::TextMap(vec![make_tycon("RentalAgreement", &dar.main.hash, vec!["DA", "RentDemo"])]);
        let expected = DamlValue::Map(DamlTextMap::from(hashmap! {
            "first".to_owned() => daml_value!({landlord: "Alice"#p, tenant: "Bob"#p, terms: "test terms"}),
            "last".to_owned() => daml_value!({landlord: "John"#p, tenant: "Paul"#p, terms: "more test terms"})
        }));
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[]` -> `M.fromList [] : Map Int`
    #[test]
    fn test_genmap_int_empty() -> DamlJsonCodecResult<()> {
        let json_value = json!([]);
        let ty = DamlType::GenMap(vec![DamlType::Int64]);
        let expected = DamlValue::GenMap(vec![].into_iter().collect());
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[["foo", 42], ["bar", 43]]` -> `M.fromList [("foo", 42), ("bar", 43)] : Map Int`
    #[test]
    fn test_genmap_string_to_int() -> DamlJsonCodecResult<()> {
        let json_value = json!([["foo", 42], ["bar", 43]]);
        let ty = DamlType::GenMap(vec![DamlType::Text, DamlType::Int64]);
        let expected = DamlValue::GenMap(
            vec![(daml_value!["foo"], daml_value![42]), (daml_value!["bar"], daml_value![43])].into_iter().collect(),
        );
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[[42, "foo"], [43, "bar"]]` -> `M.fromList [(42, "foo"), (43, "bar")] : Map Int`
    #[test]
    fn test_genmap_int_to_string() -> DamlJsonCodecResult<()> {
        let json_value = json!([[42, "foo"], [43, "bar"]]);
        let ty = DamlType::GenMap(vec![DamlType::Int64, DamlType::Text]);
        let expected = DamlValue::GenMap(
            vec![(daml_value![42], daml_value!["foo"]), (daml_value![43], daml_value!["bar"])].into_iter().collect(),
        );
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `[[{"name": "Alice", "age": 30}, "foo"], [{"name": "Bob", "age": 18}, "bar"]]` -> `M.fromList [(Person "Alice"
    /// 30, "foo"), (Person "Bob" 18, "bar")] : Map Person Text`
    #[test]
    fn test_genmap_person_to_string() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!([[{"name": "Alice", "age": 30}, "foo"], [{"name": "Bob", "age": 18}, "bar"]]);
        let key_type = make_tycon("Person", &dar.main.hash, vec!["DA", "JsonTest"]);
        let value_type = DamlType::Text;
        let ty = DamlType::GenMap(vec![key_type, value_type]);
        let expected = DamlValue::GenMap(
            vec![
                (daml_value![{name: "Alice", age: 30}], daml_value!["foo"]),
                (daml_value![{name: "Bob", age: 18}], daml_value!["bar"]),
            ]
            .into_iter()
            .collect(),
        );
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_genmap_duplicate_key_should_fail() -> DamlJsonCodecResult<()> {
        let json_value = json!([[42, "foo"], [42, "bar"]]);
        let ty = DamlType::GenMap(vec![DamlType::Int64, DamlType::Text]);
        let actual = JsonDecoder::new(&DamlArchive::default()).decode(&json_value, &ty);
        assert!(actual.is_err());
        Ok(())
    }

    /// `{"tag": "Bar", "value": 42}` -> `Bar 42 : variant Foo = Bar Int64 | Baz | Quux (Optional Int64)`
    #[test]
    fn test_variant_bar() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({"tag": "Bar", "value": 42});
        let ty = make_tycon("Foo", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = daml_value![{=>Bar 42}];
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{"tag": "Baz", "value": {}}` -> `Baz : variant Foo = Bar Int64 | Baz | Quux (Optional Int64)`
    #[test]
    fn test_variant_baz() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({"tag": "Baz", "value": {}});
        let ty = make_tycon("Foo", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = daml_value![{=>Baz}];
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{"tag": "Quux", "value": null}` -> `Quux None : variant Foo = Bar Int64 | Baz | Quux (Optional Int64)`
    #[test]
    fn test_variant_quux_none() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({"tag": "Quux", "value": null});
        let ty = make_tycon("Foo", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = daml_value![{=>Quux {?!}}];
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `{"tag": "Quux", "value": 42}` -> `Quux Some 42 : variant Foo = Bar Int64 | Baz | Quux (Optional Int64)`
    #[test]
    fn test_variant_quux_some() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!({"tag": "Quux", "value": 42});
        let ty = make_tycon("Foo", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = daml_value![{=>Quux {?=42}}];
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `Enabled` -> `Enabled : data Status = Enabled | Disabled`
    #[test]
    fn test_enum_enabled() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!("Enabled");
        let ty = make_tycon("Status", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = DamlValue::Enum(DamlEnum::new("Enabled", None));
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `Disabled` -> `Enabled : data Status = Enabled | Disabled`
    #[test]
    fn test_enum_disabled() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!("Disabled");
        let ty = make_tycon("Status", &dar.main.hash, vec!["DA", "JsonTest"]);
        let expected = DamlValue::Enum(DamlEnum::new("Disabled", None));
        let actual = decode_apply(&dar, &json_value, &ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `Unknown` -> `Enabled : data Status = Enabled | Disabled`
    #[test]
    fn test_enum_unknown_error() -> DamlJsonCodecResult<()> {
        let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)?;
        let json_value = json!("Unknown");
        let ty = make_tycon("Status", &dar.main.hash, vec!["DA", "JsonTest"]);
        let actual = decode_apply(&dar, &json_value, &ty);
        assert!(actual.is_err());
        Ok(())
    }

    fn decode_apply(dar: &DarFile, json_value: &Value, ty: &DamlType<'_>) -> DamlJsonCodecResult<DamlValue> {
        dar.apply(|arc| {
            let decoded_value = JsonDecoder::new(arc).decode(json_value, ty)?;
            Ok::<DamlValue, DamlJsonCodecError>(decoded_value)
        })?
    }

    fn make_tycon<'a>(data_name: &'a str, package_id: &'a str, module_path: Vec<&'a str>) -> DamlType<'a> {
        DamlType::TyCon(DamlTyCon::new(
            DamlTyConName::Absolute(DamlAbsoluteTyCon::new(
                Cow::from(data_name),
                Cow::from(package_id),
                Cow::from(""),
                module_path.into_iter().map(Cow::from).collect(),
            )),
            vec![],
        ))
    }
}
