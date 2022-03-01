use crate::error::DamlJsonCodecResult;
use crate::util::Required;
use bigdecimal::ToPrimitive;
use chrono::SecondsFormat;
use daml_grpc::data::value::{DamlRecord, DamlValue};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Encode a `DamlValue` as JSON.
#[derive(Debug, Default)]
pub struct JsonValueEncoder {
    encode_decimal_as_string: bool,
    encode_int64_as_string: bool,
}

impl JsonValueEncoder {
    pub const fn new(encode_decimal_as_string: bool, encode_int64_as_string: bool) -> Self {
        Self {
            encode_decimal_as_string,
            encode_int64_as_string,
        }
    }

    /// Encode a GRPC `DamlValue` as JSON `Value`.
    pub fn encode_value(&self, value: &DamlValue) -> DamlJsonCodecResult<Value> {
        self.do_encode_value(value, true)
    }

    /// Encode a GRPC `DamlRecord` as JSON `Value`.
    pub fn encode_record(&self, record: &DamlRecord) -> DamlJsonCodecResult<Value> {
        self.do_encode_record(record)
    }

    /// Recursively encode a `DamlValue` as a JSON `Value`.
    ///
    /// Here `top_level` refers to whether we are processing a value corresponding to the "top level" of a type or a
    /// nested types.  This is required to support the "shortcut" encodings for optional fields.
    ///
    /// For example given a `DamlValue` of type `DamlOptional<DamlText>` the `DamlOptional` is not considered a nested
    /// type and so `top_level` will be true whilst the contained `DamlText` is considered as a nested type and so
    /// `top_level` will be false.
    ///
    /// Note that the `DamlValue` associated with each field of a `DamlRecord` is not considered nested and so will be
    /// processed with `top_level` set to true.  If the field type contains nested data types (such as an optional)
    /// then these will behave as described above.
    ///
    /// See the [Daml LF JSON Encoding documentation](https://docs.daml.com/json-api/lf-value-specification.html) for details.
    fn do_encode_value(&self, value: &DamlValue, top_level: bool) -> DamlJsonCodecResult<Value> {
        match value {
            DamlValue::Unit => Ok(json!({})),
            DamlValue::Bool(b) => Ok(json!(b)),
            DamlValue::Int64(i) =>
                if self.encode_int64_as_string {
                    Ok(json!(format!("{}", i)))
                } else {
                    Ok(json!(i))
                },
            DamlValue::Numeric(n) =>
                if self.encode_decimal_as_string {
                    Ok(json!(format!("{}", n)))
                } else {
                    Ok(json!(n.to_f64().req()?))
                },
            DamlValue::Timestamp(timestamp) => Ok(json!(timestamp.to_rfc3339_opts(SecondsFormat::Millis, true))),
            DamlValue::Date(date) => Ok(json!(date.naive_utc().to_string())),
            DamlValue::Text(text) => Ok(json!(text)),
            DamlValue::Party(party) => Ok(json!(party.party)),
            DamlValue::ContractId(id) => Ok(json!(id.contract_id)),
            DamlValue::Optional(opt) => match opt {
                None if top_level => Ok(json!(null)),
                None => Ok(json!([])),
                Some(inner) if top_level => Ok(json!(self.do_encode_value(inner, false)?)),
                Some(inner) => Ok(json!([self.do_encode_value(inner, false)?])),
            },
            DamlValue::Record(record) => self.do_encode_record(record),
            DamlValue::List(list) => {
                let items =
                    list.iter().map(|i| self.do_encode_value(i, true)).collect::<DamlJsonCodecResult<Vec<_>>>()?;
                Ok(json!(items))
            },
            DamlValue::Map(map) => {
                let entries = map
                    .iter()
                    .map(|(k, v)| Ok((k.as_str(), self.do_encode_value(v, true)?)))
                    .collect::<DamlJsonCodecResult<HashMap<_, _>>>()?;
                Ok(json!(entries))
            },
            DamlValue::Variant(variant) => {
                let ctor = variant.constructor();
                let value = self.do_encode_value(variant.value(), true)?;
                Ok(json!({"tag": ctor, "value": value}))
            },
            DamlValue::Enum(data_enum) => Ok(json!(data_enum.constructor())),
            DamlValue::GenMap(map) => {
                let entries = map
                    .iter()
                    .map(|(k, v)| Ok((self.do_encode_value(k, true)?, self.do_encode_value(v, true)?)))
                    .collect::<DamlJsonCodecResult<Vec<(_, _)>>>()?;
                Ok(json!(entries))
            },
        }
    }

    /// Recursively encode a `DamlRecord` as a JSON `Value`.
    fn do_encode_record(&self, record: &DamlRecord) -> DamlJsonCodecResult<Value> {
        let has_labels = record.fields().iter().all(|f| f.label().is_some());
        if has_labels {
            let fields = record
                .fields()
                .iter()
                .filter(|f| !matches!(f.value(), DamlValue::Optional(None)))
                .map(|f| Ok((f.label().as_ref().req()?, self.do_encode_value(f.value(), true)?)))
                .collect::<DamlJsonCodecResult<HashMap<_, _>>>()?;
            Ok(json!(fields))
        } else {
            let fields = record
                .fields()
                .iter()
                .map(|f| self.do_encode_value(f.value(), true))
                .collect::<DamlJsonCodecResult<Vec<_>>>()?;
            Ok(json!(fields))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DamlJsonCodecResult, DamlValue, JsonValueEncoder};
    use crate::util::Required;
    use crate::value_decode::JsonTryAsExt;
    use daml::macros::daml_value;
    use daml_grpc::data::value::{DamlEnum, DamlRecord, DamlRecordField};
    use daml_grpc::data::DamlIdentifier;
    use daml_grpc::primitive_types::DamlTextMap;
    use maplit::hashmap;
    use serde_json::{json, Value};

    #[test]
    fn test_unit() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!();
        let expected = json!({});
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_bool() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!(true);
        let expected = json!(true);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_i64() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!(42);
        let expected = json!(42);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_i64_neg() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!(-42);
        let expected = json!(-42);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_i64_as_string() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!(42);
        let expected = json!("42");
        let actual = JsonValueEncoder::new(false, true).encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_i64_neg_as_string() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!(-42);
        let expected = json!("-42");
        let actual = JsonValueEncoder::new(false, true).encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_numeric() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!(1.0);
        let expected = json!(1.0);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_numeric_neg() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!(-1.0);
        let expected = json!(-1.0);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_numeric_as_string() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!(1.000_000_000_000_000);
        let expected = json!("1.000000000000000");
        let actual = JsonValueEncoder::new(true, false).encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_numeric_neg_as_string() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!(-1.000_000_000_000_000);
        let expected = json!("-1.000000000000000");
        let actual = JsonValueEncoder::new(true, false).encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_text() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!("test");
        let expected = json!("test");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_text_empty() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!("");
        let expected = json!("");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_date() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!("2019-06-18"::d);
        let expected = json!("2019-06-18");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_date_min() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!("9999-12-31"::d);
        let expected = json!("9999-12-31");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_date_max() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!("0001-01-01"::d);
        let expected = json!("0001-01-01");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_timestamp_full() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!("1990-11-09T04:30:23.1234569Z"::t);
        let expected = json!("1990-11-09T04:30:23.123Z");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_timestamp_no_sub_sec() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!("1990-11-09T04:30:23Z"::t);
        let expected = json!("1990-11-09T04:30:23.000Z");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_timestamp_no_micros() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!("1990-11-09T04:30:23.123Z"::t);
        let expected = json!("1990-11-09T04:30:23.123Z");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_timestamp_min() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!("0001-01-01T00:00:00Z"::t);
        let expected = json!("0001-01-01T00:00:00.000Z");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_timestamp_max() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!("9999-12-31T23:59:59.999999Z"::t);
        let expected = json!("9999-12-31T23:59:59.999Z");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_party() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!("Alice"::p);
        let expected = json!("Alice");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_contract_id() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!("#0:0"::c);
        let expected = json!("#0:0");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_opt_none() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({?!});
        let expected = json!(null);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_opt_int_some() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({?=42});
        let expected = json!(42);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_opt_opt_int_some_none() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({?={?!}});
        let expected = json!([]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_opt_opt_int_some_some() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({?={?=42}});
        let expected = json!([42]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_opt_opt_opt_int_some_some_none() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({?={?={?!}}});
        let expected = json!([[]]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_opt_opt_opt_int_some_some_some() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({?={?={?=42}}});
        let expected = json!([[42]]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_opt_opt_opt_opt_int_some_some_some_none() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({?={?={?={?!}}}});
        let expected = json!([[[]]]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_opt_opt_opt_opt_text_some_some_some_some() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({?={?={?={?="Test"}}}});
        let expected = json!([[["Test"]]]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_opt_unit_some() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({?=()});
        let expected = json!({});
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_empty_record() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({});
        let expected = json!({});
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_record() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({landlord: "Alice"::p, tenant: "Bob"::p, terms: "test terms"});
        let expected = json!({
             "landlord": "Alice",
             "tenant": "Bob",
             "terms": "test terms",
        });
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_record_as_array() -> DamlJsonCodecResult<()> {
        let grpc_value = DamlValue::new_record(DamlRecord::new(
            vec![
                DamlRecordField::new(None::<String>, DamlValue::new_party("Alice")),
                DamlRecordField::new(None::<String>, DamlValue::new_party("Bob")),
                DamlRecordField::new(None::<String>, DamlValue::new_text("test terms")),
            ],
            None::<DamlIdentifier>,
        ));
        let expected = json!(["Alice", "Bob", "test terms"]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `Depth1 with foo: None : data Depth1 = Depth1 with foo: Optional Int64` -> `{ }`
    ///
    /// Note that we could equally encode this as `{ "foo": null }`, both decode to the same `DamlValue`.
    #[test]
    fn test_record_depth1_omitted_or_none() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({foo: {?!}});
        let expected = json!({});
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `Depth1 with foo: Some 42 : data Depth1 = Depth1 with foo: Optional Int64` -> `{ foo: 42 }`
    #[test]
    fn test_record_depth1_some() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({foo: {?=42}});
        let expected = json!({ "foo": 42 });
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `Depth2 with foo: None : data Depth2 = Depth2 with foo: Optional (Optional Int64)` -> `{ }`
    ///
    /// Note that we could equally encode this as `{ "foo": null }`, both decode to the same `DamlValue`.
    #[test]
    fn test_record_depth2_omitted_or_none() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({foo: {?!}});
        let expected = json!({});
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `Depth2 with foo: Some None : data Depth2 = Depth2 with foo: Optional (Optional Int64)` -> `{ foo: [] }`
    #[test]
    fn test_record_depth2_some_none() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({foo: {?={?!}}});
        let expected = json!({ "foo": [] });
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// `Depth2 { foo: Some (Some 42) } : Depth2` -> `{ foo: [42] }`
    #[test]
    fn test_record_depth2_some_some() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!({foo: {?={?=42}}});
        let expected = json!({ "foo": [42] });
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_list_empty() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!([]);
        let expected = json!([]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_list_bool() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!([true, true, false, false]);
        let expected = json!([true, true, false, false]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_list_text() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!(["a", "b", "c"]);
        let expected = json!(["a", "b", "c"]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_list_opt_text() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!([{?="a"}, {?!}, {?="c"}]);
        let expected = json!(["a", null, "c"]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_list_record() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value!([{
            landlord: "Alice"::p,
            tenant: "Bob"::p,
            terms: "test terms"
        },
        {
            landlord: "John"::p,
            tenant: "Paul"::p,
            terms: "more test terms"
        }]);
        let expected = json!([{
            "landlord": "Alice",
            "tenant": "Bob",
            "terms": "test terms",
        },
        {
            "landlord": "John",
            "tenant": "Paul",
            "terms": "more test terms",
        }]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_textmap_empty() -> DamlJsonCodecResult<()> {
        let grpc_value = DamlValue::Map(vec![].into_iter().collect());
        let expected = json!({});
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_textmap_int() -> DamlJsonCodecResult<()> {
        let grpc_value = DamlValue::Map(
            vec![("foo".to_owned(), daml_value![42]), ("bar".to_owned(), daml_value![43])].into_iter().collect(),
        );
        let expected = json!({"foo": 42, "bar": 43});
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_textmap_list_int() -> DamlJsonCodecResult<()> {
        let grpc_value = DamlValue::Map(DamlTextMap::from(hashmap! {
            "foo".to_owned() => daml_value![[1, 2, 3]],
            "bar".to_owned() => daml_value![[4, 5, 6]]
        }));
        let expected = json!({"foo": [1, 2, 3], "bar": [4, 5, 6]});
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_textmap_record() -> DamlJsonCodecResult<()> {
        let grpc_value = DamlValue::Map(DamlTextMap::from(hashmap! {
            "first".to_owned() => daml_value!({landlord: "Alice"::p, tenant: "Bob"::p, terms: "test terms"}),
            "last".to_owned() => daml_value!({landlord: "John"::p, tenant: "Paul"::p, terms: "more test terms"})
        }));
        let expected = json!({
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
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_genmap_empty() -> DamlJsonCodecResult<()> {
        let grpc_value = DamlValue::GenMap(vec![].into_iter().collect());
        let expected = json!([]);
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_genmap_string_to_int() -> DamlJsonCodecResult<()> {
        let grpc_value = DamlValue::GenMap(
            vec![(daml_value!["foo"], daml_value![42]), (daml_value!["bar"], daml_value![43])].into_iter().collect(),
        );
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        let key_foo_value = find_genmap_value(&actual, &json!["foo"])?;
        let key_bar_value = find_genmap_value(&actual, &json!["bar"])?;
        assert_eq!(key_foo_value, &json![42]);
        assert_eq!(key_bar_value, &json![43]);
        Ok(())
    }

    #[test]
    fn test_genmap_int_to_string() -> DamlJsonCodecResult<()> {
        let grpc_value = DamlValue::GenMap(
            vec![(daml_value![42], daml_value!["foo"]), (daml_value![43], daml_value!["bar"])].into_iter().collect(),
        );
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        let key_42_value = find_genmap_value(&actual, &json![42])?;
        let key_43_value = find_genmap_value(&actual, &json![43])?;
        assert_eq!(key_42_value, &json!["foo"]);
        assert_eq!(key_43_value, &json!["bar"]);
        Ok(())
    }

    #[test]
    fn test_genmap_person_to_string() -> DamlJsonCodecResult<()> {
        let grpc_value = DamlValue::GenMap(
            vec![
                (daml_value![{name: "John", age: 29}], daml_value!["foo"]),
                (daml_value![{name: "Alice", age: 22}], daml_value!["bar"]),
            ]
            .into_iter()
            .collect(),
        );
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        let key_john_value = find_genmap_value(&actual, &json![{"name": "John", "age": 29}])?;
        let key_alice_value = find_genmap_value(&actual, &json![{"name": "Alice", "age": 22}])?;
        assert_eq!(key_john_value, &json!["foo"]);
        assert_eq!(key_alice_value, &json!["bar"]);
        Ok(())
    }

    #[test]
    fn test_variant_bar() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value![{=>Bar 42}];
        let expected = json!({"tag": "Bar", "value": 42});
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_variant_baz() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value![{=>Baz}];
        let expected = json!({"tag": "Baz", "value": {}});
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_variant_quuz_none() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value![{=>Quux {?!}}];
        let expected = json!({"tag": "Quux", "value": null});
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_variant_quuz_some() -> DamlJsonCodecResult<()> {
        let grpc_value = daml_value![{=>Quux {?=42}}];
        let expected = json!({"tag": "Quux", "value": 42});
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_enum_enabled() -> DamlJsonCodecResult<()> {
        let grpc_value = DamlValue::Enum(DamlEnum::new("Enabled", None));
        let expected = json!("Enabled");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_enum_disabled() -> DamlJsonCodecResult<()> {
        let grpc_value = DamlValue::Enum(DamlEnum::new("Disabled", None));
        let expected = json!("Disabled");
        let actual = JsonValueEncoder::default().encode_value(&grpc_value)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    fn find_genmap_value<'a>(genmap: &'a Value, key: &Value) -> DamlJsonCodecResult<&'a Value> {
        Ok(genmap
            .try_array()?
            .iter()
            .find_map(|item| (item.try_array().ok()?.first()? == key).then(|| item))
            .req()?
            .try_array()?
            .last()
            .req()?)
    }
}
