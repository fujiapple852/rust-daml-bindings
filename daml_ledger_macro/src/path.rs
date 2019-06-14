/// Construct a DAML data extractor function from a path expression.
///
/// This macro provides a concise DSL for constructing a DAML data extractor closure as required by
/// [`DamlRecord::extract`] and [`DamlValue::extract`].  The closure produced will have the following signature:
///
/// `Fn(&DamlRecord) -> DamlResult<R>`
///
/// The type of `R` depends on the path expression provided and may either be a reference to a [`DamlValue`] or a
/// reference to another type such as `&str`.
///
/// # Syntax
///
/// Path expressions take the following form (pseudo-regex syntax):
///
/// ``` txt
/// field ( '{' '=>' variant '}' )? ( '[' index ']' )? ( '?' )?  (  '/'  ...  )*  ( '#' type )?
/// ----- ------------------------- ------------------ --------  -  ---  ---  --  -------------
///  (1)              (2)                   (3)          (4)     |  (5)  (6)  |        (7)
/// ```
///
/// Each `field` corresponds to a labelled [`DamlRecordField`] within the [`DamlRecord`] on which the data extractor
/// is to be executed.
///
/// Syntax Items:
///
/// 1. the `field` of the current [`DamlValue::Record`] to traverse
/// 2. extract the [`DamlValue`] if `field` is a [`DamlValue::Variant`] and `varient` matches the constructor.
/// 3. extract the [`DamlValue`] from list at `index` (expression) if `field` is a [`DamlValue::List`]
/// 4. extract the [`DamlValue`] if `field` is a [`DamlValue::Optional`]
/// 5. a separator between `field` entries
/// 6. a repeat of items `(1)`, `(2)`, `(3)` & `(4)`.  Items `(5)` & `(6)` are repeated zero or many times.
/// 7. an optional type specifier
///
/// Note that any or all of `(2)`, `(3)` & `(4)` can be applied for a single `field` and will run consecutively.  i.e.
/// in the single path field `people{=>Persons}[0]?` will interpret `people` as a [`DamlValue::Variant`] with a
/// constructor of `Persons` which is a [`DamlValue::List`] which has an element at index `0` which contain an
/// [`DamlValue::Optional`] that is a [`DamlValue::Int64`].
///
/// Note that the path expression is not whitespace sensitive.
///
/// Fields which are nested [`DamlRecord`] can be traversed by chaining together multiple `field` elements delimited
/// with a `/` character.  Nesting can be of arbitrary depth (up to the default recursive macro limit).  Attempting
/// to access a non-existent field will return in an [`UnknownField`] error being returned.
///
/// List elements of `fields` which are of type [`DamlValue::List`] can be accessed by specifying a list `index`
/// expression inside `[]` braces.  A [`ListIndexOutOfRange`] error will be returned if an attempt is made to access
/// lists elements outside the available bounds.
///
/// Optional `fields` of type [`DamlValue::Optional`] can be accessed by appending a `?` character to the record
/// `field` name or list element.  If the optional is `Some` then the embedded [`DamlValue`] is extracted and
/// processing of the path expression continues.  If the optional is `None` then a [`OptionalIsNone`] error will be
/// returned.
///
/// The final `field` may optionally have a `type` specifier by appending a `#` character followed by one
/// of several supported `type` specifier codes.  If no `type` specifier code is provided then the expression will
/// return a [`DamlValue`] otherwise a type appropriate to the specifier will be returned.
///
/// If the constructor of a [`DamlValue::Variant`] does not match then [`UnexpectedVariant`] error is returned.  The
/// special variant value of `__` can be used to indicate that any variant is acceptable.  Attempting to access the
/// nested [`DamlValue`] within a variant will produce an error if the type of the item does not match the actual
/// variant type.
///
/// The supported `type` specifiers are:
///
/// | code   | name        | Rust type                 | value variant             |
/// |--------|-------------|---------------------------|----------------------------
/// | `c`    | contract id | `&str`                    | [`DamlValue::ContractId`] |
/// | `u`    | unit        | `&()`                     | [`DamlValue::Unit`]       |
/// | `p`    | party       | `&str`                    | [`DamlValue::Party`]      |
/// | `i`    | int64       | `&i64`                    | [`DamlValue::Int64`]      |
/// | `f`    | decimal     | `&BigInteger`             | [`DamlValue::Decimal`]    |
/// | `t`    | text        | `&str`                    | [`DamlValue::Text`]       |
/// | `s`    | timestamp   | `&DateTime<Utc>`          | [`DamlValue::Timestamp`]  |
/// | `b`    | boolean     | `&bool`                   | [`DamlValue::Bool`]       |
/// | `d`    | date        | `&Date<Utc>`              | [`DamlValue::Date`]       |
/// | `r`    | record      | `&DamlRecord`             | [`DamlValue::Record`]     |
/// | `l`    | list        | `&Vec<DamlValue>`         | [`DamlValue::List`]       |
/// | `v`    | variant     | `&DamlVariant`            | [`DamlValue::Variant`]    |
///
/// # Limitations
///
/// Path expressions only support labelled [`DamlRecordField`] fields.  Accessing unlabelled fields will result in
/// an [`UnknownField`] error being returned.
///
/// Accessing values from a list-of-list is not supported and therefore path expression syntax `my_list[0][1]` is not
/// valid.  To access the sublist first extract the parent list with the path expression `my_list[0]` and then apply a
/// second path expression to the resulting [`DamlValue`].  Note that a list containing [`DamlRecord`] which in turn
/// contains nested lists is supported.
///
/// All `type` specifiers return references, even for simple copy types, and so must be dereferenced as needed.
///
/// There is currently no support for [`DamlValue::Map`].
///
/// # Examples
///
/// ```
/// # use daml_ledger_api::data::value::{DamlRecord, DamlValue};
/// # use daml_ledger_api::data::{DamlResult, DamlError};
/// # use daml_ledger_macro::{daml_value, daml_path};
/// # use bigdecimal::BigDecimal;
/// # fn main() -> DamlResult<()> {
/// let value = daml_value![{
///     party: "Alice"#p,
///     trade: {
///         trade_id: 123,
///         counterparty: "Bob"#p,
///         trade_details: {
///             ticker: "GOOG",
///             prices: [1231.54, 1234.85, 1237.92]
///         },
///         order_type: {?= "MarketOrder"}
///     }
/// }];
/// assert_eq!("Alice", value.extract(daml_path![party#p])?);
/// assert_eq!(123, *value.extract(daml_path![trade/trade_id#i])?);
/// assert_eq!("Bob", value.extract(daml_path![trade/counterparty#p])?);
/// assert_eq!("GOOG", value.extract(daml_path![trade/trade_details/ticker#t])?);
/// assert_eq!(&BigDecimal::from(1234.85),
///                 value.extract(daml_path![trade/trade_details/prices[1]#f])?);
/// assert_eq!("MarketOrder", value.extract(daml_path![trade/order_type?#t])?);
/// # Ok(())
/// # }
/// ```
/// [`DamlValue::extract`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#method.extract
/// [`DamlRecord::extract`]: ../../doc/daml_ledger_api/data/value/struct.DamlRecord.html#method.extract
/// [`DamlValue`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html
/// [`DamlValue`]: daml_ledger_api::data::value::DamlValue
/// [`DamlRecord`]: ../../doc/daml_ledger_api/data/value/struct.DamlRecord.html
/// [`DamlRecordField`]: ../../doc/daml_ledger_api/data/value/struct.DamlRecordField.html
/// [`DamlValue::List`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.List
/// [`DamlValue::Optional`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Optional
/// [`DamlValue::Record`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Record
/// [`DamlValue::Variant`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Variant
/// [`DamlValue::Int64`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Int64
/// [`DamlValue::ContractId`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.ContractId
/// [`DamlValue::Unit`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Unit
/// [`DamlValue::Party`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Party
/// [`DamlValue::Decimal`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Decimal
/// [`DamlValue::Text`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Text
/// [`DamlValue::Timestamp`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Timestamp
/// [`DamlValue::Bool`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Bool
/// [`DamlValue::Date`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Date
/// [`DamlValue::Map`]: ../../doc/daml_ledger_api/data/value/enum.DamlValue.html#variant.Map
/// [`ListIndexOutOfRange`]: ../../doc/daml_ledger_api/data/enum.Error.html#variant.ListIndexOutOfRange
/// [`OptionalIsNone`]: ../../doc/daml_ledger_api/data/enum.Error.html#variant.OptionalIsNone
/// [`UnknownField`]: ../../doc/daml_ledger_api/data/enum.Error.html#variant.UnknownField
/// [`UnexpectedVariant`]: ../../doc/daml_ledger_api/data/enum.Error.html#variant.UnexpectedVariant
#[macro_export]
macro_rules! daml_path {

    // The order that these matching rules is critical so be very careful if attempting to modify or reformat this
    // macro.  The structure of this macro is split into five sections:
    //
    // 1: final-path-element matchers (i.e. "... / field")
    // 2: non-final-path-element matchers (i.e. "field / ...")
    // 3: leaf value matchers
    // 4: helper "function" matchers
    // 5: public entry point matcher
    //
    // Sections 1 & 2 define several matchers to cover cases for lists, options and variants.
    //
    // The macro has become very complex and should be rewritten as a procedural macro.

    // final path element (list + optional case)
    ( @priv $record:ident / $path:ident $( { => $variant:ident } )? [ $index:expr ] ? $( # $type:ident )? ) => {
        {
            let field_value = daml_path!(@get_record_field $record, $path);
            let variant_value = daml_path!(@get_variant_value field_value, $($variant)? )?;
            let list_item_value = daml_path!(@get_list_item variant_value, $index);
            let optional_value = list_item_value.try_optional()?.ok_or(DamlError::OptionalIsNone)?;
            daml_path!(@priv $($type)? optional_value)
        }
    };

    // final path element (list case)
    ( @priv $record:ident / $path:ident $( { => $variant:ident } )? [ $index:expr ] $( # $type:ident )? ) => {
        {
            let field_value = daml_path!(@get_record_field $record, $path);
            let variant_value = daml_path!(@get_variant_value field_value, $($variant)? )?;
            let list_item_value = daml_path!(@get_list_item variant_value, $index);
            daml_path!(@priv $($type)? list_item_value)
        }
    };

    // final path element
    ( @priv $record:ident / $path:ident $( { => $variant:ident } )? $( # $type:ident )? ) => {
        {
            let field_value = daml_path!(@get_record_field $record, $path);
            let variant_value = daml_path!(@get_variant_value field_value, $($variant)? )?;
            daml_path!(@priv $($type)? variant_value)
        }
    };

    // final path element (optional case)
    ( @priv $record:ident / $path:ident $( { => $variant:ident } )? ? $( # $type:ident )? ) => {
        {
            let field_value = daml_path!(@get_record_field $record, $path);
            let variant_value = daml_path!(@get_variant_value field_value, $($variant)? )?;
            let optional_value = variant_value.try_optional()?.ok_or(DamlError::OptionalIsNone)?;
            daml_path!(@priv $($type)? optional_value)
        }
    };

    // non-final path element (list + optional case)
    ( @priv $record:ident / $path:ident $( { => $variant:ident } )? [ $index:expr ] ? $($rest:tt)* ) => {
        {
            let field_value = daml_path!(@get_record_field $record, $path);
            let variant_value = daml_path!(@get_variant_value field_value, $($variant)? )?;
            let list_item_value = daml_path!(@get_list_item variant_value, $index);
            let optional_value = list_item_value.try_optional()?.ok_or(DamlError::OptionalIsNone)?;
            let field_as_record = &(optional_value.try_record()?);
            daml_path!( @priv field_as_record $($rest)* )
        }
    };

    // non-final path element (list case)
    ( @priv $record:ident / $path:ident $( { => $variant:ident } )? [ $index:expr ] $($rest:tt)* ) => {
        {
            let field_value = daml_path!(@get_record_field $record, $path);
            let value_from_variant = daml_path!(@get_variant_value field_value, $($variant)? )?;
            let list_item_value = daml_path!(@get_list_item value_from_variant, $index);
            let field_as_record = &(list_item_value.try_record()?);
            daml_path!( @priv field_as_record $($rest)* )
        }
    };

    // non-final path element (optional case)
    ( @priv $record:ident / $path:ident $( { => $variant:ident } )? ? $($rest:tt)* ) => {
        {
            let field_value = daml_path!(@get_record_field $record, $path);
            let variant_value = daml_path!(@get_variant_value field_value, $($variant)? )?;
            let optional_value = variant_value.try_optional()?.ok_or(DamlError::OptionalIsNone)?;
            let field_as_record = &(optional_value.try_record()?);
            daml_path!( @priv field_as_record $($rest)* )
        }
    };

    // non-final path element (special case for nested variant to resolve parsing ambiguity)
    ( @priv $record:ident / $path:ident { => $variant:ident } $($rest0:tt)* ) => {
        {
            let field_value = daml_path!(@get_record_field $record, $path);
            let variant_value = daml_path!(@get_variant_value field_value, $variant )?;
            let field_as_record = &(variant_value.try_record()?);
            daml_path!( @priv field_as_record $($rest0)* )
        }
    };

    // non-final path element
    ( @priv $record:ident / $path:ident $( { => $variant:ident } )? $($rest1:tt)* ) => {
        {
            let field_value = daml_path!(@get_record_field $record, $path);
            let variant_value = daml_path!(@get_variant_value field_value, $($variant)? )?;
            let field_as_record = &(variant_value.try_record()?);
            daml_path!( @priv field_as_record $($rest1)* )
        }
    };

    // leaf cases
    ( @priv $value:ident ) => {
        {
            let res: DamlResult<&DamlValue> = Ok($value);
            res
        }
    };
    ( @priv c $value:ident ) => {
        $value.try_contract_id()
    };
    ( @priv u $value:ident ) => {
        $value.try_unit_ref()
    };
    ( @priv p $value:ident ) => {
        $value.try_party()
    };
    ( @priv i $value:ident ) => {
        $value.try_int64_ref()
    };
    ( @priv f $value:ident ) => {
        $value.try_decimal()
    };
    ( @priv t $value:ident ) => {
        $value.try_text()
    };
    ( @priv b $value:ident ) => {
        $value.try_bool_ref()
    };
    ( @priv s $value:ident ) => {
        $value.try_timestamp_ref()
    };
    ( @priv d $value:ident ) => {
        $value.try_date_ref()
    };
    ( @priv r $value:ident ) => {
        $value.try_record()
    };
    ( @priv l $value:ident ) => {
        $value.try_list()
    };
    ( @priv v $value:ident ) => {
        $value.try_variant()
    };

    // get a named field from a record
    ( @get_record_field $record:ident, $path:ident ) => {
        $record.field(stringify!($path))?
    };

    // interpret the value as a list and extract the item from a given index
    ( @get_list_item $value:ident, $index:expr ) => {
        {
            let field_as_list = $value.try_list()?;
            let list_item_value: &DamlValue = field_as_list.get($index).ok_or(DamlError::ListIndexOutOfRange($index))?;
            list_item_value
        }
    };

    // get the value from with a variant
    ( @get_variant_value $value:ident , $variant:ident ) => {
        {
            let variant = $value.try_variant()?;
            if stringify!($variant) == "__" || variant.constructor() == stringify!($variant) {
                Ok(variant.value())
            } else {
                Err(DamlError::UnexpectedVariant(stringify!($variant).to_owned(), variant.constructor().to_owned()))
            }
        }
    };

    // get the value from with a variant (identity case)
    ( @get_variant_value $value:ident , ) => {
        {
            let res: DamlResult<_> = Ok($value);
            res
        }
    };

    // the public entry point to this macro
    ( $($rest:tt)* ) => {
        {
            use ::daml_ledger_api::data::DamlResult;
            let func: fn(&DamlRecord) -> DamlResult<&_> = | rec_ref: &DamlRecord | {
                daml_path!( @priv rec_ref / $($rest)* )
            };
            func
        }
    };
}

#[cfg(test)]
mod test {
    use crate::daml_value;
    use crate::test_util::TestResult;
    use crate::test_util::{make_date, make_timestamp};
    use bigdecimal::BigDecimal;
    use chrono::{Date, DateTime, NaiveDate, Utc};
    use daml_ledger_api::data::value::{DamlRecord, DamlValue, DamlVariant};
    use daml_ledger_api::data::DamlError;

    #[test]
    pub fn test_top_party() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("Alice", value.extract(daml_path![sender#p])?);
        Ok(())
    }

    #[test]
    pub fn test_record_top_party() -> TestResult {
        let value: DamlValue = get_test_value();
        let record: &DamlRecord = value.try_record()?;
        assert_eq!("Alice", record.extract(daml_path![sender#p])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_party() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("Sue", value.extract(daml_path![person/party#p])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_contract() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("#1:1", value.extract(daml_path![person/data/contractId#c])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_value() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(&DamlValue::new_contract_id("#1:1"), value.extract(daml_path![person / data / contractId])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_int64() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(0_i64, *value.extract(daml_path![person/data/count#i])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_bool() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(false, *value.extract(daml_path![person/data/is_true#b])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_unit() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!((), *value.extract(daml_path![person/empty#u])?);
        Ok(())
    }

    #[test]
    pub fn test_top_decimal() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(&BigDecimal::from(1.23), value.extract(daml_path![height#f])?);
        Ok(())
    }

    #[test]
    pub fn test_top_date() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(&make_date("2019-01-02")?, value.extract(daml_path![today#d])?);
        Ok(())
    }

    #[test]
    pub fn test_top_datetime() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(&make_timestamp("2019-01-02T03:45:56Z")?, value.extract(daml_path![right_now#s])?);
        Ok(())
    }

    #[test]
    pub fn test_top_optional_value() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(&DamlValue::new_int64(123), value.extract(daml_path![opt_int?])?);
        Ok(())
    }

    #[test]
    pub fn test_top_variant_value() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(&DamlValue::new_text("I'm a Foo"), value.extract(daml_path![variant_text{=>Foo}])?);
        Ok(())
    }

    #[test]
    pub fn test_top_variant() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(
            &DamlVariant::new("Foo", Box::new(DamlValue::new_text("I'm a Foo")), None),
            value.extract(daml_path![variant_text#v])?
        );
        Ok(())
    }

    #[test]
    pub fn test_nested_text() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("apple", value.extract(daml_path![person/data/fruit#t])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_record() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(4, value.extract(daml_path![person/data#r])?.fields().len());
        Ok(())
    }

    #[test]
    pub fn test_nested_list() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(3, value.extract(daml_path![person/stats#l])?.len());
        Ok(())
    }

    #[test]
    pub fn test_list_item() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("a", value.extract(daml_path![items#l])?[0].extract(daml_path![a#t])?);
        assert_eq!("b", value.extract(daml_path![items#l])?[1].extract(daml_path![b#t])?);
        Ok(())
    }

    #[test]
    pub fn test_top_list_record_item_text() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("a", value.extract(daml_path![items[0]/a#t])?);
        Ok(())
    }

    #[test]
    pub fn test_top_list_leaf_item_text() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("foo", value.extract(daml_path![simple_list[0]#t])?);
        assert_eq!("bar", value.extract(daml_path![simple_list[1]#t])?);
        Ok(())
    }

    #[test]
    pub fn test_top_list_leaf_item_value() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(&DamlValue::new_text("bar"), value.extract(daml_path![simple_list[1]])?);
        Ok(())
    }

    #[test]
    pub fn test_list_index_expression() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("bar", value.extract(daml_path![simple_list[2 - 1]#t])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_list_item_value() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(&DamlValue::new_int64(1), value.extract(daml_path![person / stats[0]])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_list_item_int64() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(1, *value.extract(daml_path![person/stats[0]#i])?);
        Ok(())
    }

    // Does not support list-of-list.  list-of-record-of-list is supported.
    //#[test]
    // pub fn test_list_in_list() -> TestResult {
    //    let value: DamlValue = get_test_value();
    //    assert_eq!(1, *value.path(daml_path![simple_list[2][0]])?);
    //    Ok(())
    //}

    #[test]
    pub fn test_list_in_list_of_record() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(99, *value.extract(daml_path![items[1]/the_list[0]#i])?);
        Ok(())
    }

    #[test]
    pub fn test_optional_int() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(123, *value.extract(daml_path![opt_int?#i])?);
        Ok(())
    }

    #[test]
    pub fn test_optional_rec() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("cat", value.extract(daml_path![opt_rec?/pet#t])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_optional_rec() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(true, *value.extract(daml_path![opt_rec?/is_cat?#b])?);
        Ok(())
    }

    #[test]
    pub fn test_list_of_optional_final() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(1, *value.extract(daml_path![list_of_opt[0]?#i])?);
        Ok(())
    }

    #[test]
    pub fn test_list_of_optional_non_final() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("a", value.extract(daml_path![list_of_opt_rec[0]?/a#t])?);
        Ok(())
    }

    #[test]
    pub fn test_top_named_variant_text() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("I'm a Foo", value.extract(daml_path![variant_text{=>Foo}#t])?);
        Ok(())
    }

    #[test]
    pub fn test_top_named_variant_value() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(&DamlValue::new_text("I'm a Foo"), value.extract(daml_path![variant_text{=>Foo}])?);
        Ok(())
    }

    #[test]
    pub fn test_top_any_variant_text() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("I'm a Foo", value.extract(daml_path![variant_text{=>__}#t])?);
        Ok(())
    }

    #[test]
    pub fn test_top_named_variant_optional_int() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(999, *value.extract(daml_path![variant_opt_int{=>Foo}?#i])?);
        Ok(())
    }

    #[test]
    pub fn test_top_named_variant_list() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("One", value.extract(daml_path![variant_list{=>Foo}[0]#t])?);
        Ok(())
    }

    #[test]
    pub fn test_top_named_variant_list_optional() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("Two", value.extract(daml_path![variant_list_opt{=>Foo}[1]?#t])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_named_variant_int() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(4, *value.extract(daml_path![other{=>Cat}/paw_count#i])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_any_variant_int() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!(4, *value.extract(daml_path![other{=>__}/paw_count#i])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_named_variant_optional_int() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("Red", value.extract(daml_path![other_opt{=>Fruit}?/color#t])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_named_variant_list() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("Blue", value.extract(daml_path![other_var_list{=>Foo}[0]/color#t])?);
        Ok(())
    }

    #[test]
    pub fn test_nested_named_variant_list_optional() -> TestResult {
        let value: DamlValue = get_test_value();
        assert_eq!("Green", value.extract(daml_path![other_var_list_opt{=>Foo}[0]?/color#t])?);
        Ok(())
    }

    #[test]
    pub fn test_top_no_result_function() {
        let value: DamlValue = get_test_value();
        assert_eq!("Alice", value.extract(daml_path![sender#p]).expect("should not fail"));
    }

    #[test]
    pub fn test_unknown_field() {
        let value: DamlValue = get_test_value();
        let result = value.extract(daml_path![unknown]);
        match result {
            Err(DamlError::UnknownField(s)) => assert_eq!("unknown", s),
            _ => panic!("expected failure"),
        }
    }

    #[test]
    pub fn test_nested_unknown_field() {
        let value: DamlValue = get_test_value();
        let result = value.extract(daml_path![person / unknown]);
        match result {
            Err(DamlError::UnknownField(s)) => assert_eq!("unknown", s),
            _ => panic!("expected failure"),
        }
    }

    #[test]
    pub fn test_wrong_type_field() {
        let value: DamlValue = get_test_value();
        let result = value.extract(daml_path![sender#i]);
        match result {
            Err(DamlError::UnexpectedType(expected, actual)) => {
                assert_eq!("Int64", expected);
                assert_eq!("Party", actual);
            },
            _ => panic!("expected failure"),
        }
    }

    #[test]
    pub fn test_list_index_out_of_range() {
        let value: DamlValue = get_test_value();
        let result = value.extract(daml_path![items[99]#i]);
        match result {
            Err(DamlError::ListIndexOutOfRange(idx)) => assert_eq!(99, idx),
            _ => panic!("expected failure"),
        }
    }

    #[test]
    pub fn test_bad_variant() {
        let value: DamlValue = get_test_value();
        let result = value.extract(daml_path![variant_text{=>Bar}#t]);
        match result {
            Err(DamlError::UnexpectedVariant(expected, actual)) => {
                assert_eq!("Bar", expected);
                assert_eq!("Foo", actual);
            },
            _ => panic!("expected failure"),
        }
    }

    fn get_test_value() -> DamlValue {
        daml_value![{
            sender: "Alice"#p,
            receiver: "Bob"#p,
            person: {
                party: "Sue"#p,
                data: {
                    count: 0,
                    fruit: "apple",
                    contractId: "#1:1"#c,
                    is_true: false
                },
                stats: [1, 2, 3],
                empty: ()
            },
            height: 1.23,
            items: [{a: "a"}, {b: "b", the_list: [99, 98, 101]}],
            simple_list: ["foo", "bar", ["text in nested list"]],
            today: "2019-01-02"#d,
            right_now: "2019-01-02T03:45:56Z"#t,
            opt_int: {?= 123},
            opt_rec: {?= {
                pet: "cat",
                is_cat: {?= true},
                food: ["ham", "eggs"]
            }},
            list_of_opt: [{?=1}, {?=2}, {?=3}],
            opt_list: {?=[1,2,3]},
            list_of_opt_rec: [
                {?= {a: "a"}},
                {?= {b: "b"}},
                {?= {c: "c"}}
            ],
            variant_text: {=>Foo "I'm a Foo"},
            variant_opt_int: {=>Foo {?= 999}},
            variant_list: {=>Foo ["One", "Two"]},
            variant_list_opt: {=>Foo [{?="One"}, {?="Two"}]},
            other: {=>Cat {
                paw_count: 4
            }},
            other_opt: {=>Fruit {?= {
                color: "Red"
            }}},
            other_var_list: {=>Foo [{color: "Blue"}]},
            other_var_list_opt: {=>Foo [{?= {color: "Green"}}]}
        }]
    }
}
