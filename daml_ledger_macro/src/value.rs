/// Construct a DAML value.
///
/// This macro provide a concise DSL for constructing an [`DamlValue`] which can be used in the creation of contracts
/// and the exercising of choices.
///
/// # Syntax
///
/// A DAML `record` is a map of `label` to `value`.  A `label` is always an identifier.  A `value` can be
/// a `literal`, an `identifier`, an `expression`, a `list`, an `optional`, a `variant` or a nested `record`.
///
/// The macro always creates a [`DamlValue`] enum which may be of any allowed enum variant.  Typically a [`DamlRecord`]
/// is being constructed and so a [`DamlValue::Record`] is produced.
///
/// DAML value pseudo-BNF grammar:
///
/// ``` bnf
/// DamlValue ::=     '{' ( label ':' DamlValue ',' )* '}'  // Record
///                 | '[' ( DamlValue ',' )* ']'            // List
///                 | '{' '=>' variant DamlValue '}'        // Variant
///                 | '{' '?=' DamlValue '}'                // Optional (Some)
///                 | '{' '?!' '}'                          // Optional (None)
///                 | identifier ('#' type)?                // String identifier
///                 | literal ('#' type)?                   // String literal
///                 | (expression) ('#' type)?              // DamlValue expression
/// ```
///
/// Note that this syntax is not whitespace sensitive.
///
/// The supported optional `type` specifiers are:
///
/// | code   | name        | type                    | example                    |
/// |--------|-------------|-------------------------| ----------------------------
/// | `p`    | party       | `DamlValue::Party`      | `"Alice"#p`                |
/// | `c`    | contract id | `DamlValue::ContractId` | `"#1:1"#c`                 |
/// | `d`    | date        | `DamlValue::Date`       | `"2019-01-01"#d`           |
/// | `t`    | timestamp   | `DamlValue::Timestamp`  | `"2019-01-01T01:23:45Z"#t` |
///
/// String literal used without a type specifier are assumed to be [`DamlValue::Text`] therefore type specifiers are
/// only required for [`DamlValue::Party`] (#p) and [`DamlValue::ContractId`] (#c).
///
/// # Limitations
///
/// Data values passed as expressions (as opposed to simple literal values or identifiers) must be placed inside
/// parentheses.
///
/// For example you may specify `my_party_name_str#p` where `my_party_name_str` is the identifier of an
/// in-scope variable containing a &str but you may not specify `get_party_name_str()#p` where `get_party_name_str()`
/// is a function (and therefore an expression).
///
/// To support such cases either use `(get_party_name_str())#p` or provide a [`DamlValue`]
/// `DamlValue::new_party(get_party_name_str())`.
///
/// There is currently no support for [`DamlValue::Map`] or [`DamlValue::Enum`].
///
/// # Examples
///
/// ```
/// # use daml_ledger_api::data::value::{DamlRecord, DamlValue, DamlVariant};
/// # use daml_ledger_api::data::{DamlResult, DamlError};
/// # use daml_ledger_macro::daml_value;
/// # fn main() -> DamlResult<()> {
/// let value = daml_value![
///     // the DamlValue being created is a DamlValue::Record
///     {
///         // party is DamlValue::Party specified as a literal with a type suffix
///         party: "Alice"#p,
///
///         // a nested DamlValue::Record
///         trade: {
///
///             // a literal DamlValue::Int64
///             trade_id: 123,
///
///             // counterparty is DamlValue::Party, provided as an expression (note the outer
///             // braces here)
///             counterparty: (DamlValue::new_party("B".to_owned() + "ob")),
///
///             // trader is a DamlValue::Party literal built from an expression
///             trader: ("Ali".to_owned() + "ce")#p,
///
///             // trade_details is a DamlValue::Variant specifying SimpleDetails which contains
///             // a nested DamlValue::Record)
///             trade_details: {=>SimpleDetails {
///
///                     // ticker is a String without type suffix and so becomes a
///                     // DamlValue::Text
///                     ticker: "GOOG",
///
///                     // prices is an DamlValue::Optional which is Some(DamlValue::List(...))
///                     prices: {?=[1231.54, 1234.85, 1237.92]},
///
///                     // description is an DamlValue::Optional which is None
///                     description: {?!},
///
///                     // side is a DamlValue::Variant specifying Buy which contains
///                     // a DamlValue::Text
///                     side: {=>Buy "MarketOrder"}
///                 }
///             }
///         }
///     }
/// ];
/// # Ok(())
/// # }
/// ```
/// [`DamlValue`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html
/// [`DamlValue::Record`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Record
/// [`DamlValue::Party`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Party
/// [`DamlValue::ContractId`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.ContractId
/// [`DamlValue::Text`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Text
/// [`DamlValue::Map`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Map
/// [`DamlValue::Enum`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Enum
/// [`DamlRecord`]: ../../doc/daml_ledger_api/data/value/struct.DamlRecord.html
#[macro_export]
macro_rules! daml_value {

    //
    // Covers DamlValue:Unit case
    //
    () => {
        DamlValue::Unit
    };

    //
    // Covers DamlValue::List case
    //
    ([ $( $value:tt $( # $type:ident )? ),* ]) => {
        {
            #[allow(unused_mut)]
            let mut list = vec![];
            $(
                list.push(daml_value!($( @priv $type )? $value));
            )*
            DamlValue::List(list)
        }
    };

    //
    // Covers DamlValue::Record case
    //
    ( { $( $label:ident : $value:tt $( # $type:ident )? ),* } ) => {
        {
            #[allow(unused_mut)]
            let mut rec_builder = daml_ledger_api::data::value::DamlRecordBuilder::new();
            $(
                rec_builder = rec_builder.add_field(stringify![$label], daml_value!($( @priv $type )? $value));
            )*
            DamlValue::new_record(rec_builder.build())
        }
    };

    //
    // Covers DamlValue::Variant case
    //
    ( { => $variant:ident $($value:tt)* } ) => {
        {
            let val = daml_value!($($value)*);
            let variant = DamlVariant::new(stringify!($variant), Box::new(val), None);
            DamlValue::new_variant(variant)
        }
    };

    //
    // Covers DamlValue::Optional (Some) case
    //
    ( { ?= $($value:tt)* } ) => {
        {
            let val = daml_value!($($value)*);
            DamlValue::new_optional(Some(val))
        }
    };

    //
    // Covers DamlValue::Optional (None) case
    //
    ( { ?! } ) => {
        {
            DamlValue::new_optional(None)
        }
    };

    //
    // Covers DamlValue::Bool, DamlValue::Text, DamlValue::Int64 & DamlValue::Decimal cases
    //
    ($prim:expr) => {
        DamlValue::from($prim)
    };

    //
    // Covers DamlValue::Party, DamlValue::ContractId, DamlValue::Timestamp & DamlValue::Date cases
    //
    ($party:tt # p) => {
        daml_value!(@priv p $party)
    };
    ($contract:tt # c) => {
        daml_value!(@priv c $contract)
    };
    ($timestamp:tt # t) => {
        daml_value!(@priv t $timestamp)
    };
    ($date:tt # d) => {
        daml_value!(@priv d $date)
    };

    //
    // These cases will only be called from within the macro and so we prefix with @priv to indicate they are private
    //
    (@priv p $party:expr) => {
        DamlValue::new_party($party)
    };
    (@priv c $contract:expr) => {
        DamlValue::new_contract_id($contract)
    };
    (@priv t $timestamp:expr) => {
        DamlValue::new_timestamp($timestamp.parse::<DateTime<Utc>>().unwrap()) // TODO shouldn't unwrap here!
    };
    (@priv d $date:expr) => {
        // TODO date parsing should be flexible and performed in DamlValue, not here!
        DamlValue::new_date(Date::<Utc>::from_utc(NaiveDate::parse_from_str($date, "%Y-%m-%d").unwrap(), Utc))
    };
}

#[cfg(test)]
mod test {
    use crate::test_util::TestResult;
    use crate::test_util::{make_date, make_timestamp};
    use bigdecimal::BigDecimal;
    use chrono::{Date, DateTime, NaiveDate, Utc};
    use daml_ledger_api::data::value::{DamlValue, DamlVariant};

    #[test]
    pub fn test_unit_value() -> TestResult {
        let value: DamlValue = daml_value![];
        assert_eq!((), value.try_unit()?);
        Ok(())
    }

    #[test]
    pub fn test_string_value() -> TestResult {
        let value: DamlValue = daml_value!["Test"];
        assert_eq!("Test", value.try_text()?);
        Ok(())
    }

    #[test]
    pub fn test_party_value() -> TestResult {
        let value: DamlValue = daml_value![DamlValue::new_party("Test")];
        assert_eq!("Test", value.try_party()?);
        Ok(())
    }

    #[test]
    pub fn test_party_literal() -> TestResult {
        let value: DamlValue = daml_value!["Test"#p];
        assert_eq!("Test", value.try_party()?);
        Ok(())
    }

    #[test]
    pub fn test_contract_id_value() -> TestResult {
        let value: DamlValue = daml_value![DamlValue::new_contract_id("123")];
        assert_eq!("123", value.try_contract_id()?);
        Ok(())
    }

    #[test]
    pub fn test_contract_id_literal() -> TestResult {
        let value: DamlValue = daml_value!["123"#c];
        assert_eq!("123", value.try_contract_id()?);
        Ok(())
    }

    #[test]
    pub fn test_int64_value() -> TestResult {
        let value: DamlValue = daml_value![42];
        assert_eq!(42, value.try_int64()?);
        Ok(())
    }

    #[test]
    pub fn test_bool_value() -> TestResult {
        let value: DamlValue = daml_value![true];
        assert_eq!(true, value.try_bool()?);
        Ok(())
    }

    #[test]
    pub fn test_decimal_value() -> TestResult {
        let value: DamlValue = daml_value![1.23];
        assert_eq!(&BigDecimal::from(1.23), value.try_decimal()?);
        Ok(())
    }

    #[test]
    pub fn test_timestamp_value() -> TestResult {
        let value: DamlValue = daml_value!["2019-01-02T03:45:56Z"#t];
        assert_eq!(make_timestamp("2019-01-02T03:45:56Z")?, value.try_timestamp()?);
        Ok(())
    }

    #[test]
    pub fn test_timestamp_literal() -> TestResult {
        let timestamp = make_timestamp("2019-01-02T03:45:56Z")?;
        let value: DamlValue = daml_value![DamlValue::new_timestamp(timestamp)];
        assert_eq!(timestamp, value.try_timestamp()?);
        Ok(())
    }

    #[test]
    pub fn test_date_literal() -> TestResult {
        let value: DamlValue = daml_value!["2019-01-02"#d];
        assert_eq!(make_date("2019-01-02")?, value.try_date()?);
        Ok(())
    }

    #[test]
    pub fn test_date_value() -> TestResult {
        let date = make_date("2019-01-02")?;
        let value: DamlValue = daml_value![DamlValue::new_date(date)];
        assert_eq!(date, value.try_date()?);
        Ok(())
    }

    #[test]
    pub fn test_empty_record() -> TestResult {
        let value: DamlValue = daml_value![{}];
        assert_eq!(0, value.try_record()?.fields().len());
        Ok(())
    }

    #[test]
    pub fn test_simple_record() -> TestResult {
        let value: DamlValue = daml_value![{
            sender: "Alice"#p,
            receiver: "Bob"#p,
            count: 0
        }];
        assert_eq!("Alice", value.try_record()?.fields()[0].value().try_party()?);
        assert_eq!("Bob", value.try_record()?.fields()[1].value().try_party()?);
        assert_eq!(0, value.try_record()?.fields()[2].value().try_int64()?);
        Ok(())
    }

    #[test]
    pub fn test_optional_none() -> TestResult {
        let value: DamlValue = daml_value![{?!}];
        assert_eq!(None, value.try_optional()?);
        Ok(())
    }

    #[test]
    pub fn test_optional_text() -> TestResult {
        let value: DamlValue = daml_value![{?="bar"}];
        assert_eq!("bar", value.try_optional()?.ok_or("not ok")?.try_text()?);
        Ok(())
    }

    #[test]
    pub fn test_optional_record() -> TestResult {
        let value: DamlValue = daml_value![{?={
            sender: "Alice"#p,
            receiver: "Bob"#p,
            count: 0
        }}];
        assert_eq!("Alice", value.try_optional()?.ok_or("not ok")?.try_record()?.fields()[0].value().try_party()?);
        assert_eq!("Bob", value.try_optional()?.ok_or("not ok")?.try_record()?.fields()[1].value().try_party()?);
        assert_eq!(0, value.try_optional()?.ok_or("not ok")?.try_record()?.fields()[2].value().try_int64()?);
        Ok(())
    }

    #[test]
    pub fn test_variant_text() -> TestResult {
        let value: DamlValue = daml_value![{=>foo "bar"}];
        assert_eq!("bar", value.try_variant()?.value().try_text()?);
        assert_eq!("foo", value.try_variant()?.constructor());
        Ok(())
    }

    #[test]
    pub fn test_variant_party_literal() -> TestResult {
        let value: DamlValue = daml_value![{=>PartyId "Bob"#p}];
        assert_eq!("Bob", value.try_variant()?.value().try_party()?);
        assert_eq!("PartyId", value.try_variant()?.constructor());
        Ok(())
    }

    #[test]
    pub fn test_variant_record() -> TestResult {
        let value: DamlValue = daml_value![{=>Variant1 {
            sender: "Alice"#p,
            receiver: "Bob"#p,
            count: 0
        }}];
        assert_eq!("Alice", value.try_variant()?.value().try_record()?.fields()[0].value().try_party()?);
        assert_eq!("Bob", value.try_variant()?.value().try_record()?.fields()[1].value().try_party()?);
        assert_eq!(0, value.try_variant()?.value().try_record()?.fields()[2].value().try_int64()?);
        assert_eq!("Variant1", value.try_variant()?.constructor());
        Ok(())
    }

    #[test]
    pub fn test_nested_variant() -> TestResult {
        let value: DamlValue = daml_value![{
            data: {=>SomeVariant "data from SomeVariant"}
        }];
        assert_eq!("data from SomeVariant", value.try_record()?.fields()[0].value().try_variant()?.value().try_text()?);
        assert_eq!("SomeVariant", value.try_record()?.fields()[0].value().try_variant()?.constructor());
        Ok(())
    }

    #[test]
    pub fn test_nested_variant_record() -> TestResult {
        let value: DamlValue = daml_value![{
            data: {=>foo {a: "test"}}
        }];
        assert_eq!(
            "test",
            value.try_record()?.fields()[0].value().try_variant()?.value().try_record()?.fields()[0]
                .value()
                .try_text()?
        );
        Ok(())
    }

    #[test]
    pub fn test_nested_variant_record_optional() -> TestResult {
        let value: DamlValue = daml_value![{
            sender: "Alice"#p,
            receiver: "Bob"#p,
            count: 0,
            data: {=>foo {
                mand_text: "test",
                opt_int: {?= 0},
                opt_node: {?!}
            }}
        }];
        assert_eq!(
            "test",
            value.try_record()?.fields()[3].value().try_variant()?.value().try_record()?.fields()[0]
                .value()
                .try_text()?
        );
        assert_eq!(
            0,
            value.try_record()?.fields()[3].value().try_variant()?.value().try_record()?.fields()[1]
                .value()
                .try_optional()?
                .ok_or("not ok")?
                .try_int64()?
        );
        assert_eq!(
            None,
            value.try_record()?.fields()[3].value().try_variant()?.value().try_record()?.fields()[2]
                .value()
                .try_optional()?
        );
        Ok(())
    }

    #[test]
    pub fn test_nested_record() -> TestResult {
        let value: DamlValue = daml_value![{
            sender: "Alice"#p,
            receiver: "Bob"#p,
            data: {
                count: 0,
                fruit: "apple",
                contractId: "#1:1"#c
            }
        }];
        assert_eq!("Alice", value.try_record()?.fields()[0].value().try_party()?);
        assert_eq!("Bob", value.try_record()?.fields()[1].value().try_party()?);
        assert_eq!(0, value.try_record()?.fields()[2].value().try_record()?.fields()[0].value().try_int64()?);
        assert_eq!("apple", value.try_record()?.fields()[2].value().try_record()?.fields()[1].value().try_text()?);
        assert_eq!(
            "#1:1",
            value.try_record()?.fields()[2].value().try_record()?.fields()[2].value().try_contract_id()?
        );
        Ok(())
    }

    #[test]
    pub fn test_empty_list() -> TestResult {
        let value: DamlValue = daml_value!([]);
        assert_eq!(0, value.try_list()?.len());
        Ok(())
    }

    #[test]
    pub fn test_simple_list() -> TestResult {
        let value: DamlValue = daml_value!([1, 2, 3]);
        assert_eq!(1, value.try_list()?[0].try_int64()?);
        assert_eq!(3, value.try_list()?.len());
        Ok(())
    }

    #[test]
    pub fn test_list_of_empty_records() -> TestResult {
        let value: DamlValue = daml_value!([{}, {}]);
        assert_eq!(2, value.try_list()?.len());
        assert_eq!(0, value.try_list()?[1].try_record()?.fields().len());
        Ok(())
    }

    #[test]
    pub fn test_list_of_record() -> TestResult {
        let value: DamlValue = daml_value!(
            [{
                foo: "bar",
                bar: "foo"
            }]
        );
        assert_eq!(1, value.try_list()?.len());
        assert_eq!("foo", value.try_list()?[0].try_record()?.fields()[1].value().try_text()?);
        Ok(())
    }

    #[test]
    pub fn test_list_of_mixed() -> TestResult {
        let value: DamlValue = daml_value!(
            [{
                foo: "bar",
                bar: "foo"
            },
            "Bob"#p,
            10
            ]
        );
        assert_eq!(3, value.try_list()?.len());
        assert_eq!("foo", value.try_list()?[0].try_record()?.fields()[1].value().try_text()?);
        assert_eq!("Bob", value.try_list()?[1].try_party()?);
        assert_eq!(10, value.try_list()?[2].try_int64()?);
        Ok(())
    }

    #[test]
    pub fn test_expressions() -> TestResult {
        let value: DamlValue = daml_value![{
            name: (String::from("John")),
            age: (21 + 3)
        }];
        assert_eq!("John", value.try_record()?.fields()[0].value().try_text()?);
        assert_eq!(24, value.try_record()?.fields()[1].value().try_int64()?);
        Ok(())
    }

    #[test]
    pub fn test_from_variables() -> TestResult {
        let party = "John";
        let age = 21;
        let dob = "1999-12-31";
        let home_address = "somewhere";
        let value: DamlValue = daml_value![{
            party: party#p,
            age: age,
            dob: dob#d,
            sex: "Male",
            address: home_address
        }];
        assert_eq!("John", value.try_record()?.fields()[0].value().try_party()?);
        assert_eq!(21, value.try_record()?.fields()[1].value().try_int64()?);
        assert_eq!(make_date("1999-12-31")?, value.try_record()?.fields()[2].value().try_date()?);
        assert_eq!("Male", value.try_record()?.fields()[3].value().try_text()?);
        assert_eq!("somewhere", value.try_record()?.fields()[4].value().try_text()?);
        Ok(())
    }
}
