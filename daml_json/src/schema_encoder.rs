use std::collections::BTreeMap;

use itertools::Itertools;
use serde_json::json;
use serde_json::Value;

use daml_lf::element::{
    DamlArchive, DamlData, DamlEnum, DamlField, DamlTyCon, DamlTyConName, DamlType, DamlTypeVarWithKind, DamlVar,
    DamlVariant,
};

use crate::error::DamlJsonSchemaCodecError::NotSerializableDamlType;
use crate::error::{DamlJsonSchemaCodecError, DamlJsonSchemaCodecResult};
use crate::schema_data::{
    DamlJsonSchemaBool, DamlJsonSchemaContractId, DamlJsonSchemaDate, DamlJsonSchemaDecimal, DamlJsonSchemaEnum,
    DamlJsonSchemaGenMap, DamlJsonSchemaGenMapItems, DamlJsonSchemaInt64, DamlJsonSchemaList, DamlJsonSchemaOptional,
    DamlJsonSchemaOptionalNonTopLevel, DamlJsonSchemaOptionalNonTopLevelOneOf, DamlJsonSchemaOptionalTopLevel,
    DamlJsonSchemaParty, DamlJsonSchemaRecord, DamlJsonSchemaRecordAsArray, DamlJsonSchemaRecordAsObject,
    DamlJsonSchemaText, DamlJsonSchemaTextMap, DamlJsonSchemaTimestamp, DamlJsonSchemaUnit, DamlJsonSchemaVariant,
    DamlJsonSchemaVariantArm,
};
use crate::util::AsSingleSliceExt;
use crate::util::Required;

/// The JSON schema version.
const SCHEMA_VERSION: &str = "https://json-schema.org/draft/2020-12/schema";

/// Control which JSON schemas should include a `$schema` property.
#[derive(Debug)]
pub enum RenderSchema {
    /// Do not render the `$schema` property for any schemas
    None,
    /// Render the `$schema` property for Daml data (Record, Template, Enum & Variant) schemas only.
    Data,
    /// Render the `$schema` property for all schemas.
    All,
}

impl Default for RenderSchema {
    fn default() -> Self {
        Self::Data
    }
}

/// Control which JSON schemas should include a `$schema` property.
#[derive(Debug)]
pub enum RenderTitle {
    /// Do not render the `title` property for any schemas
    None,
    /// Render the `title` property for Daml data (Record, Template, Enum & Variant) schemas only.
    Data,
    /// Render the `title` property for all schemas.
    All,
}

impl Default for RenderTitle {
    fn default() -> Self {
        Self::All
    }
}

/// Control whether nested `DamlTyCon` are referenced or inlined.
///
/// If `Inline` mode is set then the encoder will attempt to emit the target data type nested under the parent data
/// type.
///
/// If `Reference` mode is set then the encoder will attempt to emit an absolute reference to a data type from a given
/// `prefix`.  In this mode it is assumed the the target data type will be emitted elsewhere and made available under
/// the `prefix`.
///
/// It is not possible to emit every possible `DamlTyCon` in the requested mode and so there are some specific rules
/// which apply based on the requested mode, whether the target data type is recursive and whether the target type
/// expects type parameters.
///
/// - Recursive: Indicates that the `DamlTyCon` resolves to a data type which (directly or indirectly) contains itself
/// as a field or type argument.
///
/// - Type Parameters:  Indicates that the `DamlTyCon` resolves to a data type which has type one or more parameters
/// that must be resolved before that data type can be emitted.
///
/// The following table enumerates how a `DamlTyCon` will be encoded for all possible cases:
///
/// | Mode      | Recursive? | Type params? | Encoding                                                     |
/// |-----------|------------|--------------|--------------------------------------------------------------|
/// | Inline    | No         | No           | 1 - Encode target type inline                                |
/// | Inline    | No         | Yes          | 2 - Encode target type inline (with resolved type arguments) |
/// | Inline    | Yes        | No           | 3 - Encode to accept any object                              |
/// | Inline    | Yes        | Yes          | 4 - Encode to accept any object                              |
/// | Reference | No         | No           | 5 - Encode as reference to target type                       |
/// | Reference | No         | Yes          | 6 - Encode target type inline (fallback to #2)               |
/// | Reference | Yes        | No           | 7 - Encode as reference to target type                       |
/// | Reference | Yes        | Yes          | 8 - Encode as accept any object (no fallback possible)       |
///
/// Cases 1, 2, 5 & 7 are straightforward, whereas cases 3, 4, 6 & 8 are more complex:
///
/// * Cases 3 & 4:
///
/// If `Inline` mode is chosen and the `DamlTyCon` resolves to a data type which is recursive then the emitter emits a
/// JSON schema object which matches any JSON type:
///
/// For example, given:
///
/// ```daml
/// data Rec = Rec with foo: Text, bar: Rec
/// ```
///
/// The data type `Rec` includes itself recursively and so cannot be emitted `Inline` and will instead be emitted as
/// follows:
///
/// ```json
/// {
///    "title": "Any (Rec)",
///    "comment": "inline recursive data types cannot be represented"
/// }
/// ```
///
/// * Case 6:
///
/// If `Reference` mode is chosen and the `DamlTyCon` resolves to a data type which expects type parameters we
/// do not emit a reference as the fully resolved target data type is unknown.  In this case the emitter will
/// `fallback` to `Inline` mode.
///
/// For example, given:
///
/// ```daml
/// data Bottom a = Bottom with bottom: a
/// data Middle = Middle with middle: Bottom Int
/// ```
///
/// Attempting to emit the `middle: Bottom Int` field in `Reference` mode (with a prefix of `#/components/schemas/`)
/// cannot emit the below reference as this does not account for the type parameter applied to `Bottom`, of which there
/// are infinitely many:
///
/// ```json
/// {
///   "$ref": "#/components/schemas/Bottom"
/// }
/// ```
///
/// Instead, the schema for `Bottom Int` will be emitted `Inline`.
///
/// * Case 8:
///
/// Case 8 is similar to case 6, however, the `DamlTyCon` resolves to a data type which is also recursive.  In this
/// case we cannot 'fallback' to `Inline` mode as recursive types cannot be inlined.  The emitter therefore emits a
/// JSON schema object which matches any JSON type.
///
/// For example, given:
///
/// ```daml
/// data TopRec a = TopRec with top: TopRec a
/// ```
///
/// The structure `TopRec` is both recursive and has a type parameter and therefore cannot be emitted as a `$ref` nor
/// can it 'fallback' to `Inline` mode.
#[derive(Debug)]
pub enum ReferenceMode {
    /// Inline nested `DamlTyCon`.
    Inline,
    /// Reference nested `DamlTyCon` by `$ref` from `prefix`.
    Reference {
        prefix: String,
    },
}

impl Default for ReferenceMode {
    fn default() -> Self {
        Self::Inline
    }
}

/// JSON schema encoder configuration.
#[derive(Debug, Default)]
pub struct SchemaEncoderConfig {
    render_schema: RenderSchema,
    render_title: RenderTitle,
    reference_mode: ReferenceMode,
}

impl SchemaEncoderConfig {
    pub fn new(render_schema: RenderSchema, render_title: RenderTitle, reference_mode: ReferenceMode) -> Self {
        Self {
            render_schema,
            render_title,
            reference_mode,
        }
    }
}

/// Encode a `DamlArchive` as a JSON schema.
///
/// Generate [JSON Schema](https://json-schema.org/) from Daml LF using the `draft/2020-12/schema` version of the
/// schema.
#[derive(Debug)]
pub struct JsonSchemaEncoder<'a> {
    arc: &'a DamlArchive<'a>,
    config: SchemaEncoderConfig,
}

impl<'a> JsonSchemaEncoder<'a> {
    /// Create a Json schema encoder for a given `DamlArchive` with the default `SchemaEncoderConfig`.
    pub fn new(arc: &'a DamlArchive<'a>) -> Self {
        Self {
            arc,
            config: SchemaEncoderConfig::default(),
        }
    }

    /// Create a Json schema encoder for a given `DamlArchive` with the given `SchemaEncoderConfig`.
    pub fn new_with_config(arc: &'a DamlArchive<'a>, config: SchemaEncoderConfig) -> Self {
        Self {
            arc,
            config,
        }
    }

    /// Encode a `DamlType` as a JSON schema.
    pub fn encode_type(&self, ty: &DamlType<'_>) -> DamlJsonSchemaCodecResult<Value> {
        self.do_encode_type(ty, true, &[], &[])
    }

    /// Encode a `DamlData` as a JSON schema.
    pub fn encode_data(&self, data: &DamlData<'_>) -> DamlJsonSchemaCodecResult<Value> {
        (data.serializable() && data.type_params().is_empty())
            .then(|| self.do_encode_data(data, &[]))
            .unwrap_or_else(|| Err(NotSerializableDamlType(data.name().to_owned())))
    }

    /// Encode a Daml `Unit` type as JSON schema.
    ///
    /// A Daml LF `Unit` type is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#unit) as an empty
    /// JSON object and matches the following JSON schema:
    ///
    /// ```json
    /// {
    ///   "type": "object",
    ///   "title": "Unit",
    ///   "additionalProperties": false
    /// }
    /// ```
    fn encode_unit(&self) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaUnit {
            schema: self.schema_if_all(),
            title: self.title_if_all("Unit"),
            ty: "object",
            additional_properties: false,
        })?)
    }

    /// Encode a Daml `Bool` type as JSON schema.
    ///
    /// A Daml LF `Bool` type is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#bool) as a JSON
    /// `boolean` and matches the following JSON schema:
    ///
    /// ```json
    /// {
    ///   "type": "boolean",
    ///   "title": "Bool"
    /// }
    /// ```
    fn encode_bool(&self) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaBool {
            schema: self.schema_if_all(),
            title: self.title_if_all("Bool"),
            ty: "boolean",
        })?)
    }

    /// Encode a Daml `Text` type as JSON schema.
    ///
    /// A Daml LF `Text` type is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#text) as a JSON
    /// `string` and matches the following JSON schema:
    ///
    /// ```json
    /// {
    ///   "type": "string",
    ///   "title": "Text"
    /// }
    /// ```
    fn encode_text(&self) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaText {
            schema: self.schema_if_all(),
            title: self.title_if_all("Text"),
            ty: "string",
        })?)
    }

    /// Encode a Daml `Party` type as JSON schema.
    ///
    /// A Daml LF `Party` type is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#party) as a
    /// JSON `string` and matches the following JSON schema:
    ///
    /// ```json
    /// {
    ///   "type": "string",
    ///   "title": "Party"
    /// }
    /// ```
    fn encode_party(&self) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaParty {
            schema: self.schema_if_all(),
            title: self.title_if_all("Party"),
            ty: "string",
        })?)
    }

    /// Encode a Daml `ContractId` type as JSON schema.
    ///
    /// A Daml LF `ContractId` type is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#contractid)
    /// as a JSON `string` and matches the following JSON schema:
    ///
    /// ```json
    /// {
    ///   "type": "string",
    ///   "title": "ContractId"
    /// }
    /// ```
    fn encode_contract_id(&self) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaContractId {
            schema: self.schema_if_all(),
            title: self.title_if_all("ContractId"),
            ty: "string",
        })?)
    }

    /// Encode a Daml `Date` type as JSON schema.
    ///
    /// A Daml LF `Date` type is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#date) as a JSON
    /// `string` and matches the following JSON schema:
    ///
    /// ```json
    /// {
    ///   "type": "string",
    ///   "title": "Date"
    /// }
    /// ```
    fn encode_date(&self) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaDate {
            schema: self.schema_if_all(),
            title: self.title_if_all("Date"),
            ty: "string",
        })?)
    }

    /// Encode a Daml `Timestamp` type as JSON schema.
    ///
    /// A Daml LF `Timestamp` type is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#date) as a
    /// JSON `string` and matches the following JSON schema:
    ///
    /// ```json
    /// {
    ///   "type": "string",
    ///   "title": "Timestamp"
    /// }
    /// ```
    fn encode_timestamp(&self) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaTimestamp {
            schema: self.schema_if_all(),
            title: self.title_if_all("Timestamp"),
            ty: "string",
        })?)
    }

    /// Encode a Daml `Int64` type as JSON schema.
    ///
    /// A Daml LF `Int64` type can be [encoded](https://docs.daml.com/json-api/lf-value-specification.html#int64) as
    /// either a JSON `integer` or as a JSON `string` and matches the following JSON schema:
    ///
    /// ```json
    /// {
    ///   "type": "string",
    ///   "title": "Int64"
    /// }
    /// ```
    fn encode_int64(&self) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaInt64 {
            schema: self.schema_if_all(),
            title: self.title_if_all("Int64"),
            ty: json!(["integer", "string"]),
        })?)
    }

    /// Encode a Daml `Decimal` type as JSON schema.
    ///
    /// A Daml LF `Decimal` type can be [encoded](https://docs.daml.com/json-api/lf-value-specification.html#decimal)
    /// as either a JSON `number` or as a JSON `string` and matches the following JSON schema:
    ///
    /// ```json
    /// {
    ///   "type": "string",
    ///   "title": "Decimal"
    /// }
    /// ```
    fn encode_decimal(&self) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaDecimal {
            schema: self.schema_if_all(),
            title: self.title_if_all("Decimal"),
            ty: json!(["number", "string"]),
        })?)
    }

    /// Encode a Daml `List` type as JSON schema.
    ///
    /// A Daml LF `List a` type is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#list) as a JSON
    /// array and matches the following JSON schema where each array item is the JSON schema encoding of the type `a`:
    ///
    /// ```json
    /// {
    ///   "type": "array",
    ///   "title": "List",
    ///   "items": {
    ///     "type": "..."
    ///   }
    /// }
    /// ```
    fn encode_list(&self, items: Value) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaList {
            schema: self.schema_if_all(),
            title: self.title_if_all("List"),
            ty: "array",
            items,
        })?)
    }

    /// Encode a Daml `TextMap` type as JSON schema.
    ///
    /// A Daml LF `TextMap a` type is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#textmap) as
    /// a JSON object and matches the following JSON schema where the value of each entry is the JSON schema encoding
    /// of the type `a`:
    ///
    /// ```json
    /// {
    ///   "type": "object",
    ///   "title": "TextMap",
    ///   "additionalProperties": {
    ///     "type": "..."
    ///   }
    /// }
    /// ```
    ///
    /// > ⓘ Note: it is not possible to enforce the uniqueness of object properties in the JSON schema (see
    /// [here](https://github.com/json-schema-org/json-schema-vocabularies/issues/22) for details) and so it is
    /// assumed that the uniqueness will be enforced by the JSON parser, though the JSON specification does not require
    /// them to do so.
    fn encode_textmap(&self, additional_properties: Value) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaTextMap {
            schema: self.schema_if_all(),
            title: self.title_if_all("TextMap"),
            ty: "object",
            additional_properties,
        })?)
    }

    /// Encode a Daml `GenMap` type as JSON schema.
    ///
    /// A Daml LF `GenMap k v` type is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#genmap) as
    /// a JSON array of any length and matches the following JSON schema where each array item is a JSON array of
    /// length 2 where the first item is the JSON schema encoding of the type `k` and the second item is the JSON
    /// schema encoding of the type `v`:
    ///
    /// ```json
    /// {
    ///   "title": "GenMap",
    ///   "type": "array",
    ///   "items": {
    ///     "type": "array",
    ///     "items": [
    ///       {
    ///         "type": "..."
    ///       },
    ///       {
    ///         "type": "..."
    ///       }
    ///     ],
    ///     "minItems": 2,
    ///     "maxItems": 2,
    ///     "additionalItems": false
    ///   }
    /// }
    /// ```
    ///
    /// > ⓘ Note: The LF encoding specification states that _"any duplicate keys will cause the map to be treated as
    /// invalid"_ however this cannot be enforced by the JSON schema for this array of `[key, val]` arrays.
    fn encode_genmap(&self, ty_key: Value, ty_value: Value) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaGenMap {
            schema: self.schema_if_all(),
            title: self.title_if_all("GenMap"),
            ty: "array",
            items: DamlJsonSchemaGenMapItems {
                ty: "array",
                items: [ty_key, ty_value],
                min_items: 2,
                max_items: 2,
                additional_items: false,
            },
        })?)
    }

    /// Encode a Daml `Optional` type as JSON schema.
    ///
    /// A top-level Daml LF `Optional a` type is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#optional)
    /// as a JSON `null` (`None` case) or the JSON schema encoding of the type `a` (`Some a` case) and matches the
    /// following JSON schema:
    ///
    /// ```json
    /// {
    ///   "title": "Optional",
    ///   "oneOf": [
    ///     {
    ///       "type": "null"
    ///     },
    ///     {
    ///       "type": "..."
    ///     }
    ///   ]
    /// }
    /// ```
    ///
    /// A nested Daml LF `Optional a` type is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#optional)
    /// as an empty JSON `array` (`None` case), or a JSON array of length one where the sole array item is the encoding
    /// of the type `a` (`Some a` case) and matches the following JSON schema:
    ///
    /// ```json
    /// {
    ///   "title": "Optional (depth > 1)",
    ///   "oneOf": [
    ///     {
    ///       "type": "array",
    ///       "minItems": 0,
    ///       "maxItems": 0
    ///     },
    ///     {
    ///       "type": "array",
    ///       "items": {
    ///         "type": "..."
    ///       },
    ///       "minItems": 1,
    ///       "maxItems": 1
    ///     }
    ///   ]
    /// }
    /// ```
    ///
    /// > ⓘ Note: Top-level optional fields may be excluded from the JSON object encoding of Daml `Record` types, see
    /// the section on Daml `Record` below.
    ///
    /// > ⓘ Note: Nested optionals refers to non-top-level the optional such as the optional in parentheses in the
    /// type `Optional (Optional a)`
    fn encode_optional(&self, nested: Value, top_level: bool) -> DamlJsonSchemaCodecResult<Value> {
        if top_level {
            Ok(serde_json::to_value(DamlJsonSchemaOptional::TopLevel(DamlJsonSchemaOptionalTopLevel {
                schema: self.schema_if_all(),
                title: self.title_if_all("Optional"),
                one_of: [json!({ "type": "null" }), nested],
            }))?)
        } else {
            Ok(serde_json::to_value(DamlJsonSchemaOptional::NonTopLevel(DamlJsonSchemaOptionalNonTopLevel {
                schema: self.schema_if_all(),
                title: self.title_if_all("Optional (depth > 1)"),
                one_of: [
                    DamlJsonSchemaOptionalNonTopLevelOneOf {
                        ty: "array",
                        items: None,
                        min_items: 0,
                        max_items: 0,
                    },
                    DamlJsonSchemaOptionalNonTopLevelOneOf {
                        ty: "array",
                        items: Some(nested),
                        min_items: 1,
                        max_items: 1,
                    },
                ],
            }))?)
        }
    }

    /// Encode a Daml `Record` data type as JSON schema.
    ///
    /// A Daml LF `Record` type can be [encoded](https://docs.daml.com/json-api/lf-value-specification.html#record) as
    /// either a JSON object, or a JSON list and matches the following JSON schema:
    ///
    /// ```json
    /// {
    ///   "$schema": "https://json-schema.org/draft/2020-12/schema",
    ///   "title": "Record (... name ...)",
    ///   "oneOf": [
    ///     {
    ///       "title": "Record ...",
    ///       "type": "object",
    ///       "properties": {
    ///         "field1": {
    ///           "type": "..."
    ///         },
    ///         "field2": {
    ///           "type": "..."
    ///         }
    ///       },
    ///       "additionalProperties": false,
    ///       "required": [
    ///         "list",
    ///         "of",
    ///         "required",
    ///         "properties"
    ///       ]
    ///     },
    ///     {
    ///       "title": "Record ...",
    ///       "type": "array",
    ///       "items": [
    ///         {
    ///           "type": "..."
    ///         },
    ///         {
    ///           "type": "..."
    ///         }
    ///       ],
    ///       "minItems": "...",
    ///       "maxItems": "...",
    ///       "additionalItems": false
    ///     }
    ///   ]
    /// }
    /// ```
    ///
    /// > ⓘ Note: For the JSON object encoding, optional fields may be omitted and so only mandatory fields will be
    /// included in the `required` property list.
    ///
    /// > ⓘ Note: For the JSON list encoding, all fields will be included, and the order is significant. The
    /// `minItems` and `maxItems`will be set to reflect the number of fields on the record.
    fn do_encode_record(
        &self,
        name: &str,
        fields: &[DamlField<'_>],
        type_params: &[DamlTypeVarWithKind<'a>],
        type_args: &[DamlType<'_>],
    ) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaRecord {
            schema: self.schema_if_data_or_all(),
            title: self.title_if_data_or_all(&format!("Record ({})", name)),
            one_of: [
                self.do_encode_record_object(name, fields, type_args, type_params)?,
                self.do_encode_record_list(name, fields, type_params, type_args)?,
            ],
        })?)
    }

    /// Encode a Daml `Variant` data type as JSON schema.
    ///
    /// A Daml LF `Variant` is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#variant) as one of
    /// several JSON `object`, each containing a `tag` and a `value` and matches the following JSON schema, where the
    /// `tag` is a JSON `string` with a single possible value and `value` is the type contained with a given variant
    /// arm:
    ///
    /// ```json
    /// {
    ///   "$schema": "https://json-schema.org/draft/2020-12/schema",
    ///   "title": "Variant (... name ...)",
    ///   "oneOf": [
    ///     {
    ///       "title": "Variant ...",
    ///       "type": "object",
    ///       "properties": {
    ///         "tag": {
    ///           "type": "string",
    ///           "enum": [
    ///             "tag1"
    ///           ]
    ///         },
    ///         "value": {
    ///           "type": "..."
    ///         }
    ///       },
    ///       "additionalProperties": false,
    ///       "required": [ "tag", "value" ]
    ///     },
    ///     {
    ///       "title": "Variant ...",
    ///       "type": "object",
    ///       "properties": {
    ///         "tag": {
    ///           "type": "string",
    ///           "enum": [
    ///             "tag2"
    ///           ]
    ///         },
    ///         "value": {
    ///           "type": "..."
    ///         }
    ///       },
    ///       "additionalProperties": false,
    ///       "required": [ "tag", "value" ]
    ///     }
    ///   ]
    /// }
    /// ```
    fn encode_variant(
        &self,
        variant: &DamlVariant<'_>,
        type_params: &[DamlTypeVarWithKind<'a>],
        type_args: &[DamlType<'_>],
    ) -> DamlJsonSchemaCodecResult<Value> {
        let all_arms = variant
            .fields()
            .iter()
            .map(|field| self.encode_variant_arm(variant.name(), field, type_params, type_args))
            .collect::<DamlJsonSchemaCodecResult<Vec<_>>>()?;
        Ok(serde_json::to_value(DamlJsonSchemaVariant {
            schema: self.schema_if_data_or_all(),
            title: self.title_if_data_or_all(&format!("Variant ({})", variant.name())),
            one_of: all_arms,
        })?)
    }

    /// Encode a Daml `Enum` data type as JSON schema.
    ///
    /// A Daml LF `Enum` is [encoded](https://docs.daml.com/json-api/lf-value-specification.html#enum) as JSON `string`
    /// with a defined set of possible enum values and matches the following JSON schema:
    ///
    /// ```json
    /// {
    ///   "$schema": "https://json-schema.org/draft/2020-12/schema",
    ///   "title": "Enum ...",
    ///   "type": "string",
    ///   "enum": [
    ///     "Possible",
    ///     "Enum",
    ///     "Values"
    ///   ]
    /// }
    /// ```
    fn encode_enum(&self, data_enum: &DamlEnum<'_>) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaEnum {
            schema: self.schema_if_data_or_all(),
            title: self.title_if_data_or_all(&format!("Enum ({})", data_enum.name())),
            ty: "string",
            data_enum: data_enum.constructors().collect::<Vec<&str>>(),
        })?)
    }

    fn do_encode_type(
        &self,
        ty: &DamlType<'_>,
        top_level: bool,
        type_params: &[DamlTypeVarWithKind<'_>],
        type_args: &[DamlType<'_>],
    ) -> DamlJsonSchemaCodecResult<Value> {
        match ty {
            DamlType::Unit => self.encode_unit(),
            DamlType::Bool => self.encode_bool(),
            DamlType::Text => self.encode_text(),
            DamlType::ContractId(_) => self.encode_contract_id(),
            DamlType::Party => self.encode_party(),
            DamlType::Timestamp => self.encode_timestamp(),
            DamlType::Date => self.encode_date(),
            DamlType::Int64 => self.encode_int64(),
            DamlType::Numeric(_) => self.encode_decimal(),
            DamlType::List(tys) =>
                self.encode_list(self.do_encode_type(tys.as_single()?, true, type_params, type_args)?),
            DamlType::TextMap(tys) =>
                self.encode_textmap(self.do_encode_type(tys.as_single()?, true, type_params, type_args)?),
            DamlType::GenMap(tys) => self.encode_genmap(
                self.do_encode_type(tys.first().req()?, true, type_params, type_args)?,
                self.do_encode_type(tys.last().req()?, true, type_params, type_args)?,
            ),
            DamlType::Optional(nested) => self
                .encode_optional(self.do_encode_type(nested.as_single()?, false, type_params, type_args)?, top_level),
            DamlType::TyCon(tycon) => self.encode_tycon(tycon),
            DamlType::BoxedTyCon(tycon) => self.encode_boxed_tycon(tycon),
            DamlType::Var(v) => self.do_encode_type(
                Self::resolve_type_var(type_params, type_args, v)?,
                top_level,
                type_params,
                type_args,
            ),
            DamlType::Nat(_)
            | DamlType::Arrow
            | DamlType::Any
            | DamlType::TypeRep
            | DamlType::Update
            | DamlType::Scenario
            | DamlType::Forall(_)
            | DamlType::Struct(_)
            | DamlType::Syn(_)
            // TODO
            | DamlType::AnyException
            | DamlType::GeneralError
            | DamlType::ArithmeticError
            | DamlType::ContractError
            | DamlType::Bignumeric
            | DamlType::RoundingMode => Err(DamlJsonSchemaCodecError::UnsupportedDamlType(ty.name().to_owned())),
        }
    }

    /// Encode a `DamlTyCon`.
    ///
    /// This covers cases 1, 2, 5 & 6 in the `ReferenceMode` documentation.
    fn encode_tycon(&self, tycon: &DamlTyCon<'_>) -> DamlJsonSchemaCodecResult<Value> {
        match &self.config.reference_mode {
            ReferenceMode::Inline => {
                // cases 1 & 2
                let data = self.resolve_tycon(tycon)?;
                self.do_encode_data(data, tycon.type_arguments())
            },
            ReferenceMode::Reference {
                prefix,
            } => {
                let data = self.resolve_tycon(tycon)?;
                if data.type_params().is_empty() {
                    // case 5
                    Ok(Self::encode_reference(prefix, tycon.tycon()))
                } else {
                    // case 6
                    self.do_encode_data(data, tycon.type_arguments())
                }
            },
        }
    }

    /// Encode a `DamlTyCon` which recursively (directly or indirectly) references itself.
    ///
    /// This covers cases 3, 4, 7 & 8 in the `ReferenceMode` documentation.
    fn encode_boxed_tycon(&self, tycon: &DamlTyCon<'_>) -> DamlJsonSchemaCodecResult<Value> {
        match &self.config.reference_mode {
            ReferenceMode::Inline => {
                // cases 3 & 4
                Ok(Self::encode_inline_recursive(&tycon.tycon().to_string()))
            },
            ReferenceMode::Reference {
                prefix,
            } => {
                let data = self.resolve_tycon(tycon)?;
                if data.type_params().is_empty() {
                    // case 7
                    Ok(Self::encode_reference(prefix, tycon.tycon()))
                } else {
                    // case 8
                    Ok(Self::encode_reference_recursive_with_type_params(&tycon.tycon().to_string()))
                }
            },
        }
    }

    fn do_encode_data(&self, data: &DamlData<'_>, type_args: &[DamlType<'_>]) -> DamlJsonSchemaCodecResult<Value> {
        data.serializable()
            .then(|| match data {
                DamlData::Template(template) =>
                    self.do_encode_record(template.name(), template.fields(), &[], type_args),
                DamlData::Record(record) =>
                    self.do_encode_record(record.name(), record.fields(), record.type_params(), type_args),
                DamlData::Variant(variant) => self.encode_variant(variant, variant.type_params(), type_args),
                DamlData::Enum(data_enum) => self.encode_enum(data_enum),
            })
            .unwrap_or_else(|| Err(NotSerializableDamlType(data.name().to_owned())))
    }

    fn do_encode_record_object(
        &self,
        name: &str,
        fields: &[DamlField<'_>],
        type_args: &[DamlType<'_>],
        type_params: &[DamlTypeVarWithKind<'a>],
    ) -> DamlJsonSchemaCodecResult<Value> {
        let fields_map = fields
            .iter()
            .map(|field| {
                self.do_encode_type(field.ty(), true, type_params, type_args).map(|json_val| (field.name(), json_val))
            })
            .collect::<DamlJsonSchemaCodecResult<BTreeMap<&str, Value>>>()?;
        let opt_fields = fields
            .iter()
            .filter_map(|field| match Self::is_optional_field(field, type_args, type_params) {
                Ok(b) if !b => Some(Ok(field.name())),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<DamlJsonSchemaCodecResult<Vec<_>>>()?;
        Ok(serde_json::to_value(DamlJsonSchemaRecordAsObject {
            ty: "object",
            title: self.title_if_all(&format!("Record ({})", name)),
            properties: fields_map,
            additional_properties: false,
            required: opt_fields,
        })?)
    }

    fn do_encode_record_list(
        &self,
        name: &str,
        fields: &[DamlField<'_>],
        type_params: &[DamlTypeVarWithKind<'a>],
        type_args: &[DamlType<'_>],
    ) -> DamlJsonSchemaCodecResult<Value> {
        let fields_list = fields
            .iter()
            .map(|field| self.do_encode_type(field.ty(), true, type_params, type_args))
            .collect::<DamlJsonSchemaCodecResult<Vec<Value>>>()?;
        let field_names = fields.iter().map(DamlField::name).join(", ");
        let item_count = fields_list.len();
        Ok(serde_json::to_value(DamlJsonSchemaRecordAsArray {
            ty: "array",
            title: self.title_if_all(&format!("Record ({}, fields = [{}])", name, field_names)),
            items: fields_list,
            min_items: item_count,
            max_items: item_count,
            additional_items: false,
        })?)
    }

    fn encode_variant_arm(
        &self,
        name: &str,
        daml_field: &DamlField<'_>,
        type_params: &[DamlTypeVarWithKind<'a>],
        type_args: &[DamlType<'_>],
    ) -> DamlJsonSchemaCodecResult<Value> {
        Ok(serde_json::to_value(DamlJsonSchemaVariantArm {
            ty: "object",
            title: self.title_if_all(&format!("Variant ({}, tag={})", name, daml_field.name())),
            properties: json!(
               {
                 "tag": { "type": "string", "enum": [daml_field.name()] },
                 "value": self.do_encode_type(daml_field.ty(), true, type_params, type_args)?
               }
            ),
            required: vec!["tag", "value"],
            additional_properties: false,
        })?)
    }

    ///
    fn encode_reference(prefix: &str, tycon: &DamlTyConName<'_>) -> Value {
        json!({ "$ref": format!("{}{}.{}", prefix, tycon.module_path().join("."), tycon.data_name()) })
    }

    /// Inline recursive data types cannot be represented and so we emit a schema object which matches anything.
    fn encode_inline_recursive(name: &str) -> Value {
        json!(
            {
                "title": format!("Any ({})", name),
                "comment": "inline recursive data types cannot be represented"
            }
        )
    }

    /// Reference recursive data types with type parameters cannot be represented and so we emit a schema object which
    /// matches anything.
    fn encode_reference_recursive_with_type_params(name: &str) -> Value {
        json!(
            {
                "title": format!("Any ({})", name),
                "comment": "recursive data types with type parameters cannot be represented"
            }
        )
    }

    /// Resolve a `DamlTyCon` to a `DamlData` from the archive.
    fn resolve_tycon(&self, tycon: &DamlTyCon<'_>) -> DamlJsonSchemaCodecResult<&DamlData<'_>> {
        self.arc.data_by_tycon(tycon).ok_or_else(|| DamlJsonSchemaCodecError::DataNotFound(tycon.tycon().to_string()))
    }

    /// Resolve a `DamlVar` to a specific `DamlType` from the current type arguments by matching the position of the var
    /// in the type parameters.
    fn resolve_type_var<'arg>(
        type_params: &[DamlTypeVarWithKind<'_>],
        type_args: &'arg [DamlType<'arg>],
        var: &DamlVar<'_>,
    ) -> DamlJsonSchemaCodecResult<&'arg DamlType<'arg>> {
        let index = type_params
            .iter()
            .position(|h| h.var() == var.var())
            .ok_or_else(|| DamlJsonSchemaCodecError::TypeVarNotFoundInArgs(var.var().to_string()))?;
        type_args.get(index).ok_or_else(|| DamlJsonSchemaCodecError::TypeVarNotFoundInParams(var.var().to_string()))
    }

    /// Determine if a given field is `DamlType::Optional`, or a `DamlType::Var` that resolves to a
    /// `DamlType::Optional`.
    fn is_optional_field(
        field: &DamlField<'_>,
        type_args: &[DamlType<'_>],
        type_params: &[DamlTypeVarWithKind<'a>],
    ) -> DamlJsonSchemaCodecResult<bool> {
        match field.ty() {
            DamlType::Optional(_) => Ok(true),
            DamlType::Var(var) =>
                Ok(matches!(Self::resolve_type_var(type_params, type_args, var)?, DamlType::Optional(_))),
            _ => Ok(false),
        }
    }

    fn schema_if_all(&self) -> Option<&'static str> {
        matches!(self.config.render_schema, RenderSchema::All).then(|| SCHEMA_VERSION)
    }

    fn schema_if_data_or_all(&self) -> Option<&'static str> {
        matches!(self.config.render_schema, RenderSchema::Data | RenderSchema::All).then(|| SCHEMA_VERSION)
    }

    fn title_if_all<'t>(&self, title: &'t str) -> Option<&'t str> {
        matches!(self.config.render_title, RenderTitle::All).then(|| title)
    }

    fn title_if_data_or_all<'t>(&self, title: &'t str) -> Option<&'t str> {
        matches!(self.config.render_title, RenderTitle::Data | RenderTitle::All).then(|| title)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use assert_json_diff::assert_json_eq;
    use jsonschema::JSONSchema;
    use once_cell::sync::OnceCell;

    use daml_lf::DarFile;

    use super::*;

    static TESTING_TYPES_DAR_PATH: &str = "../resources/testing_types_sandbox/TestingTypes-latest.dar";

    #[macro_export]
    macro_rules! get_expected {
        ($name : literal) => {
            serde_json::from_str::<Value>(include_str!(concat!("../test_resources/json_schema/", $name)))
        };
    }

    #[test]
    fn test_unit() -> DamlJsonSchemaCodecResult<()> {
        let ty = DamlType::Unit;
        let expected = get_expected!("test_unit.json")?;
        let actual = JsonSchemaEncoder::new(&DamlArchive::default()).encode_type(&ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_text() -> DamlJsonSchemaCodecResult<()> {
        let ty = DamlType::Text;
        let expected = get_expected!("test_text.json")?;
        let actual = JsonSchemaEncoder::new(&DamlArchive::default()).encode_type(&ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_party() -> DamlJsonSchemaCodecResult<()> {
        let ty = DamlType::Party;
        let expected = get_expected!("test_party.json")?;
        let actual = JsonSchemaEncoder::new(&DamlArchive::default()).encode_type(&ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_int64() -> DamlJsonSchemaCodecResult<()> {
        let ty = DamlType::Int64;
        let expected = get_expected!("test_int64.json")?;
        let actual = JsonSchemaEncoder::new(&DamlArchive::default()).encode_type(&ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_numeric() -> DamlJsonSchemaCodecResult<()> {
        let ty = DamlType::Numeric(Box::new(DamlType::Nat(18)));
        let expected = get_expected!("test_numeric.json")?;
        let actual = JsonSchemaEncoder::new(&DamlArchive::default()).encode_type(&ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_bool() -> DamlJsonSchemaCodecResult<()> {
        let ty = DamlType::Bool;
        let expected = get_expected!("test_bool.json")?;
        let actual = JsonSchemaEncoder::new(&DamlArchive::default()).encode_type(&ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_contract_id() -> DamlJsonSchemaCodecResult<()> {
        let ty = DamlType::ContractId(None);
        let expected = get_expected!("test_contract_id.json")?;
        let actual = JsonSchemaEncoder::new(&DamlArchive::default()).encode_type(&ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_timestamp() -> DamlJsonSchemaCodecResult<()> {
        let ty = DamlType::Timestamp;
        let expected = get_expected!("test_timestamp.json")?;
        let actual = JsonSchemaEncoder::new(&DamlArchive::default()).encode_type(&ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_date() -> DamlJsonSchemaCodecResult<()> {
        let ty = DamlType::Date;
        let expected = get_expected!("test_date.json")?;
        let actual = JsonSchemaEncoder::new(&DamlArchive::default()).encode_type(&ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// Optional Int64
    #[test]
    fn test_optional_int64() -> DamlJsonSchemaCodecResult<()> {
        let ty = DamlType::Optional(vec![DamlType::Int64]);
        let expected = get_expected!("test_optional_int64.json")?;
        let actual = JsonSchemaEncoder::new(&DamlArchive::default()).encode_type(&ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// Optional (Optional Int64)
    #[test]
    fn test_optional_optional_int64() -> DamlJsonSchemaCodecResult<()> {
        let ty = DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Int64])]);
        let expected = get_expected!("test_optional_optional_int64.json")?;
        let actual = JsonSchemaEncoder::new(&DamlArchive::default()).encode_type(&ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    /// Optional (Optional (Optional Int64))
    #[test]
    fn test_optional_optional_optional_int64() -> DamlJsonSchemaCodecResult<()> {
        let ty = DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Optional(vec![DamlType::Int64])])]);
        let expected = get_expected!("test_optional_optional_optional_int64.json")?;
        let actual = JsonSchemaEncoder::new(&DamlArchive::default()).encode_type(&ty)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_list_of_text() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "VariantExamples"], "RecordArgument");
        let expected = get_expected!("test_list_of_text.json")?;
        let actual = JsonSchemaEncoder::new(arc).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_text_map_of_int64() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "MapTest"], "Bar");
        let expected = get_expected!("test_text_map_of_int64.json")?;
        let actual = JsonSchemaEncoder::new(arc).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_gen_map_of_int_text() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "MapTest"], "Foo");
        let expected = get_expected!("test_gen_map_of_int_text.json")?;
        let actual = JsonSchemaEncoder::new(arc).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_record() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "JsonTest"], "Person");
        let expected = get_expected!("test_record.json")?;
        let actual = JsonSchemaEncoder::new(arc).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_template() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "PingPong"], "Ping");
        let expected = get_expected!("test_template.json")?;
        let actual = JsonSchemaEncoder::new(arc).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_enum() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "Vehicle"], "SimpleColor");
        let expected = get_expected!("test_enum.json")?;
        let actual = JsonSchemaEncoder::new(arc).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_variant() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "Shape"], "Color");
        let expected = get_expected!("test_variant.json")?;
        let actual = JsonSchemaEncoder::new(arc).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_optional_depth1() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "JsonTest"], "Depth1");
        let expected = get_expected!("test_optional_depth1.json")?;
        let actual = JsonSchemaEncoder::new(arc).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_optional_depth2() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "JsonTest"], "Depth2");
        let expected = get_expected!("test_optional_depth2.json")?;
        let actual = JsonSchemaEncoder::new(arc).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    /// Covers case 1 from `ReferenceMode` (inline, non-recursive, no type parameters)
    #[test]
    fn test_reference_mode_case_1() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "JsonTest"], "PersonMap");
        let expected = get_expected!("test_reference_mode_case_1.json")?;
        let config = SchemaEncoderConfig::new(RenderSchema::default(), RenderTitle::default(), ReferenceMode::Inline);
        let actual = JsonSchemaEncoder::new_with_config(arc, config).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    /// Covers case 2 from `ReferenceMode` (inline, non-recursive, with type parameters)
    #[test]
    fn test_reference_mode_case_2() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "JsonTest"], "OPerson");
        let expected = get_expected!("test_reference_mode_case_2.json")?;
        let config = SchemaEncoderConfig::new(RenderSchema::default(), RenderTitle::default(), ReferenceMode::Inline);
        let actual = JsonSchemaEncoder::new_with_config(arc, config).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    /// Covers case 3 from `ReferenceMode` (inline, recursive, no type parameters)
    #[test]
    fn test_reference_mode_case_3() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "JsonTest"], "Rec");
        let expected = get_expected!("test_reference_mode_case_3.json")?;
        let config = SchemaEncoderConfig::new(RenderSchema::default(), RenderTitle::default(), ReferenceMode::Inline);
        let actual = JsonSchemaEncoder::new_with_config(arc, config).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    /// Covers case 4 from `ReferenceMode` (inline, recursive, with type parameters)
    #[test]
    fn test_reference_mode_case_4() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "GenericTypes"], "PatternRecord");
        let expected = get_expected!("test_reference_mode_case_4.json")?;
        let config = SchemaEncoderConfig::new(RenderSchema::default(), RenderTitle::default(), ReferenceMode::Inline);
        let actual = JsonSchemaEncoder::new_with_config(arc, config).encode_type(&ty)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    /// Covers case 5 from `ReferenceMode` (reference, non-recursive, no type parameters)
    #[test]
    fn test_reference_mode_case_5() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let data = arc.data(arc.main_package_id(), &["DA", "JsonTest"], "PersonMap").req()?;
        let expected = get_expected!("test_reference_mode_case_5.json")?;
        let config =
            SchemaEncoderConfig::new(RenderSchema::default(), RenderTitle::default(), ReferenceMode::Reference {
                prefix: "#/components/schemas/".to_string(),
            });
        let actual = JsonSchemaEncoder::new_with_config(arc, config).encode_data(data)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    /// Covers case 6 from `ReferenceMode` (reference, non-recursive, with type parameters)
    #[test]
    fn test_reference_mode_case_6() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let data = arc.data(arc.main_package_id(), &["DA", "JsonTest"], "Middle").req()?;
        let expected = get_expected!("test_reference_mode_case_6.json")?;
        let config =
            SchemaEncoderConfig::new(RenderSchema::default(), RenderTitle::default(), ReferenceMode::Reference {
                prefix: "#/components/schemas/".to_string(),
            });
        let actual = JsonSchemaEncoder::new_with_config(arc, config).encode_data(data)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    /// Covers case 7 from `ReferenceMode` (reference, recursive, no type parameters)
    #[test]
    fn test_reference_mode_case_7() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let data = arc.data(arc.main_package_id(), &["DA", "JsonTest"], "Rec").req()?;
        let expected = get_expected!("test_reference_mode_case_7.json")?;
        let config =
            SchemaEncoderConfig::new(RenderSchema::default(), RenderTitle::default(), ReferenceMode::Reference {
                prefix: "#/components/schemas/".to_string(),
            });
        let actual = JsonSchemaEncoder::new_with_config(arc, config).encode_data(data)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    /// Covers case 8 from `ReferenceMode` (reference, recursive, with type parameters)
    #[test]
    fn test_reference_mode_case_8() -> DamlJsonSchemaCodecResult<()> {
        let arc = daml_archive();
        let data = arc.data(arc.main_package_id(), &["DA", "JsonTest"], "TopRec").req()?;
        let expected = get_expected!("test_reference_mode_case_8.json")?;
        let config =
            SchemaEncoderConfig::new(RenderSchema::default(), RenderTitle::default(), ReferenceMode::Reference {
                prefix: "#/components/schemas/".to_string(),
            });
        let actual = JsonSchemaEncoder::new_with_config(arc, config).encode_data(data)?;
        assert_json_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_fail_for_non_serializable_record() -> Result<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "HigherKindTest"], "HigherKindedData");
        match JsonSchemaEncoder::new(arc).encode_type(&ty) {
            Err(DamlJsonSchemaCodecError::NotSerializableDamlType(s)) if s == "HigherKindedData" => Ok(()),
            Err(e) => panic!("expected different error: {}", e.to_string()),
            _ => panic!("expected error"),
        }
    }

    #[test]
    fn test_fail_for_generic_missing_type_arg() -> Result<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "JsonTest"], "Oa");
        match JsonSchemaEncoder::new(arc).encode_type(&ty) {
            Err(DamlJsonSchemaCodecError::TypeVarNotFoundInParams(s)) if s == "a" => Ok(()),
            Err(e) => panic!("expected different error: {}", e.to_string()),
            _ => panic!("expected error"),
        }
    }

    // Test the generated JSON schema against various sample JSON values.
    //

    #[test]
    fn test_validate_unit() -> Result<()> {
        validate_schema_match(&DamlType::Unit, &json!({}))
    }

    #[test]
    fn test_validate_unit_unexpected_property() -> Result<()> {
        validate_schema_no_match(&DamlType::Unit, &json!({ "unexpected_key": "unexpected_value" }))
    }

    #[test]
    fn test_validate_int64_as_integer() -> Result<()> {
        validate_schema_match(&DamlType::Int64, &json!(42))
    }

    #[test]
    fn test_validate_int64_as_string() -> Result<()> {
        validate_schema_match(&DamlType::Int64, &json!("42"))
    }

    #[test]
    fn test_validate_int64_invalid() -> Result<()> {
        validate_schema_no_match(&DamlType::Int64, &json!(3.4111))
    }

    #[test]
    fn test_validate_text() -> Result<()> {
        validate_schema_match(&DamlType::Text, &json!("test"))
    }

    #[test]
    fn test_validate_text_invalid() -> Result<()> {
        validate_schema_no_match(&DamlType::Text, &json!(42))
    }

    #[test]
    fn test_validate_party() -> Result<()> {
        validate_schema_match(&DamlType::Party, &json!("Alice"))
    }

    #[test]
    fn test_validate_party_invalid() -> Result<()> {
        validate_schema_no_match(&DamlType::Party, &json!(1.234))
    }

    #[test]
    fn test_validate_contract_id() -> Result<()> {
        validate_schema_match(&DamlType::ContractId(None), &json!("#1:0"))
    }

    #[test]
    fn test_validate_contract_id_invalid() -> Result<()> {
        validate_schema_no_match(&DamlType::ContractId(None), &json!({}))
    }

    #[test]
    fn test_validate_bool_true() -> Result<()> {
        validate_schema_match(&DamlType::Bool, &json!(true))
    }

    #[test]
    fn test_validate_bool_false() -> Result<()> {
        validate_schema_match(&DamlType::Bool, &json!(false))
    }

    #[test]
    fn test_validate_bool_invalid() -> Result<()> {
        validate_schema_no_match(&DamlType::Bool, &json!(0))
    }

    #[test]
    fn test_validate_numeric_with_decimal() -> Result<()> {
        validate_schema_match(&DamlType::Numeric(Box::new(DamlType::Nat(18))), &json!(9.99))
    }

    #[test]
    fn test_validate_numeric_with_integer() -> Result<()> {
        validate_schema_match(&DamlType::Numeric(Box::new(DamlType::Nat(18))), &json!(42))
    }

    #[test]
    fn test_validate_numeric_with_decimal_string() -> Result<()> {
        validate_schema_match(&DamlType::Numeric(Box::new(DamlType::Nat(18))), &json!("3.14"))
    }

    #[test]
    fn test_validate_numeric_with_integer_string() -> Result<()> {
        validate_schema_match(&DamlType::Numeric(Box::new(DamlType::Nat(18))), &json!("42"))
    }

    #[test]
    fn test_validate_numeric_invalid() -> Result<()> {
        validate_schema_no_match(&DamlType::Numeric(Box::new(DamlType::Nat(18))), &json!([1, 2, 3]))
    }

    #[test]
    fn test_validate_date() -> Result<()> {
        validate_schema_match(&DamlType::Date, &json!("2021-05-14"))
    }

    #[test]
    fn test_validate_bad_date() -> Result<()> {
        validate_schema_match(&DamlType::Date, &json!("the schema only validates that this is a string"))
    }

    #[test]
    fn test_validate_date_invalid() -> Result<()> {
        validate_schema_no_match(&DamlType::Date, &json!(1234))
    }

    #[test]
    fn test_validate_timestamp() -> Result<()> {
        validate_schema_match(&DamlType::Timestamp, &json!("1990-11-09T04:30:23.1234569Z"))
    }

    #[test]
    fn test_validate_bad_timestamp() -> Result<()> {
        validate_schema_match(&DamlType::Timestamp, &json!("the schema only validates that this is a string"))
    }

    #[test]
    fn test_validate_timestamp_invalid() -> Result<()> {
        validate_schema_no_match(&DamlType::Timestamp, &json!({"foo": 42}))
    }

    #[test]
    fn test_validate_list_of_int() -> Result<()> {
        validate_schema_match(&DamlType::List(vec![DamlType::Int64]), &json!([1, 2, 3, 42]))
    }

    #[test]
    fn test_validate_list_of_text() -> Result<()> {
        validate_schema_match(&DamlType::List(vec![DamlType::Text]), &json!(["this", "is", "a", "test"]))
    }

    #[test]
    fn test_validate_list_invalid_mixed_types() -> Result<()> {
        validate_schema_no_match(&DamlType::List(vec![DamlType::Text]), &json!(["foo", 42, "bar"]))
    }

    #[test]
    fn test_validate_textmap_of_int64() -> Result<()> {
        validate_schema_match(&DamlType::TextMap(vec![DamlType::Int64]), &json!({"key1": 1, "key2": 2}))
    }

    #[test]
    fn test_validate_textmap_of_int64_empty() -> Result<()> {
        validate_schema_match(&DamlType::TextMap(vec![DamlType::Int64]), &json!({}))
    }

    #[test]
    fn test_validate_textmap_of_int64_invalid() -> Result<()> {
        validate_schema_no_match(&DamlType::TextMap(vec![DamlType::Int64]), &json!({"key1": {}}))
    }

    /// The JSON schema does _not_ validate the uniqueness of keys in a `TextMap` and the JSON implementation used for
    /// this test does not enforce it either.
    #[test]
    fn test_validate_textmap_of_int64_duplicate_key() -> Result<()> {
        validate_schema_match(&DamlType::TextMap(vec![DamlType::Int64]), &json!({"key1": 1, "key1": 2}))
    }

    #[test]
    fn test_validate_genmap_of_int64_to_text() -> Result<()> {
        validate_schema_match(
            &DamlType::GenMap(vec![DamlType::Int64, DamlType::Text]),
            &json!([[101, "foo"], [102, "bar"]]),
        )
    }

    #[test]
    fn test_validate_genmap_of_person_to_text() -> Result<()> {
        let arc = daml_archive();
        let person_ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "JsonTest"], "Person");
        let ty = DamlType::GenMap(vec![person_ty, DamlType::Text]);
        let instance = json!(
            [[{"name": "Alice", "age": 10}, "Alice is 10"], [{"name": "Bob", "age": 6}, "Bob is 6"]]
        );
        validate_schema_for_arc_match(arc, &ty, &instance)
    }

    /// { foo: 42 }     -->  Oa { foo: Some 42 }        : Oa Int
    #[test]
    fn test_validate_generic_opt_int_some() -> Result<()> {
        let arc = daml_archive();
        let ty =
            DamlType::make_tycon_with_args(arc.main_package_id(), &["DA", "JsonTest"], "Oa", vec![DamlType::Int64]);
        let instance = json!({ "foo": 42 });
        validate_schema_for_arc_match(arc, &ty, &instance)
    }

    /// { }             -->  Oa { foo: None }           : Oa Int
    #[test]
    fn test_validate_generic_opt_int_none() -> Result<()> {
        let arc = daml_archive();
        let ty =
            DamlType::make_tycon_with_args(arc.main_package_id(), &["DA", "JsonTest"], "Oa", vec![DamlType::Int64]);
        let instance = json!({});
        validate_schema_for_arc_match(arc, &ty, &instance)
    }

    /// { foo: [42] }   -->  Oa { foo: Some (Some 42) } : Oa (Optional Int)
    #[test]
    fn test_validate_generic_opt_opt_int_some() -> Result<()> {
        let arc = daml_archive();
        let ty =
            DamlType::make_tycon_with_args(arc.main_package_id(), &["DA", "JsonTest"], "Oa", vec![DamlType::Optional(
                vec![DamlType::Int64],
            )]);
        let instance = json!({ "foo": [42] });
        validate_schema_for_arc_match(arc, &ty, &instance)
    }

    /// { foo: [] }     -->  Oa { foo: Some None }      : Oa (Optional Int)
    #[test]
    fn test_validate_generic_opt_opt_int_none() -> Result<()> {
        let arc = daml_archive();
        let ty =
            DamlType::make_tycon_with_args(arc.main_package_id(), &["DA", "JsonTest"], "Oa", vec![DamlType::Optional(
                vec![DamlType::Int64],
            )]);
        let instance = json!({ "foo": [] });
        validate_schema_for_arc_match(arc, &ty, &instance)
    }

    #[test]
    fn test_validate_genmap_of_int64_to_text_empty() -> Result<()> {
        validate_schema_match(&DamlType::GenMap(vec![DamlType::Int64, DamlType::Text]), &json!([]))
    }

    #[test]
    fn test_validate_genmap_of_int64_to_text_broken() -> Result<()> {
        validate_schema_no_match(&DamlType::GenMap(vec![DamlType::Int64, DamlType::Text]), &json!([[101]]))
    }

    #[test]
    fn test_validate_genmap_of_int64_to_text_invalid() -> Result<()> {
        validate_schema_no_match(&DamlType::GenMap(vec![DamlType::Int64, DamlType::Text]), &json!(123))
    }

    #[test]
    fn test_validate_variant() -> Result<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "VariantExamples"], "AllVariantTypes");
        let instance = json!(
            {
              "tag": "TupleStructListOfPrimitive", "value": [1, 2, 3]
            }
        );
        validate_schema_for_arc_match(arc, &ty, &instance)
    }

    #[test]
    fn test_validate_variant_unit_value() -> Result<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "VariantExamples"], "AllVariantTypes");
        let instance = json!(
            {
              "tag": "NoArgument", "value": {}
            }
        );
        validate_schema_for_arc_match(arc, &ty, &instance)
    }

    #[test]
    fn test_validate_variant_unknown_tag() -> Result<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "VariantExamples"], "AllVariantTypes");
        let instance = json!(
            {
              "tag": "UnknownTag", "value": {}
            }
        );
        validate_schema_for_arc_no_match(arc, &ty, &instance)
    }

    #[test]
    fn test_validate_variant_no_tag_or_value() -> Result<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "VariantExamples"], "AllVariantTypes");
        let instance = json!({});
        validate_schema_for_arc_no_match(arc, &ty, &instance)
    }

    #[test]
    fn test_validate_variant_no_value() -> Result<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "VariantExamples"], "AllVariantTypes");
        let instance = json!(
            {
              "tag": "NoArgument"
            }
        );
        validate_schema_for_arc_no_match(arc, &ty, &instance)
    }

    #[test]
    fn test_validate_complex_as_object_omit_opt_field() -> Result<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "Nested"], "NestedTemplate");
        let instance = json!(
            {
              "list_of_opt_of_map_of_data": [null, {"key": { "my_bool": true }}],
              "map_of_data_to_text": [[{ "my_bool": true }, "text"]],
              "owner": "me"
            }
        );
        validate_schema_for_arc_match(arc, &ty, &instance)
    }

    #[test]
    fn test_validate_complex_as_array() -> Result<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "Nested"], "NestedTemplate");
        let instance = json!(
            [
              "me",
              null,
              [null, {"key": { "my_bool": true }}],
              [[{ "my_bool": true }, "text"]],
            ]
        );
        validate_schema_for_arc_match(arc, &ty, &instance)
    }

    #[test]
    fn test_validate_complex_invalid_missing_mand_property() -> Result<()> {
        let arc = daml_archive();
        let ty = DamlType::make_tycon(arc.main_package_id(), &["DA", "Nested"], "NestedTemplate");
        let instance = json!(
            {
              "list_of_opt_of_map_of_data": [null, {"key": { "my_bool": true }}],
              "map_of_data_to_text": [[{ "my_bool": true }, "text"]]
            }
        );
        validate_schema_for_arc_no_match(arc, &ty, &instance)
    }

    fn validate_schema_match(ty: &DamlType<'_>, instance: &Value) -> Result<()> {
        do_validate_schema(&DamlArchive::default(), ty, instance, true)
    }

    fn validate_schema_no_match(ty: &DamlType<'_>, instance: &Value) -> Result<()> {
        do_validate_schema(&DamlArchive::default(), ty, instance, false)
    }

    fn validate_schema_for_arc_match(arc: &DamlArchive<'_>, ty: &DamlType<'_>, instance: &Value) -> Result<()> {
        do_validate_schema(arc, ty, instance, true)
    }

    fn validate_schema_for_arc_no_match(arc: &DamlArchive<'_>, ty: &DamlType<'_>, instance: &Value) -> Result<()> {
        do_validate_schema(arc, ty, instance, false)
    }

    fn do_validate_schema(arc: &DamlArchive<'_>, ty: &DamlType<'_>, instance: &Value, matches: bool) -> Result<()> {
        let schema = JsonSchemaEncoder::new(arc).encode_type(ty)?;
        let compiled = JSONSchema::compile(&schema)?;
        let result = compiled.validate(instance);
        assert_eq!(matches, result.is_ok());
        Ok(())
    }

    fn daml_archive() -> &'static DamlArchive<'static> {
        static INSTANCE: OnceCell<DamlArchive<'_>> = OnceCell::new();
        INSTANCE.get_or_init(|| {
            let dar = DarFile::from_file(TESTING_TYPES_DAR_PATH)
                .unwrap_or_else(|_| panic!("dar file not found: {}", TESTING_TYPES_DAR_PATH));
            dar.to_owned_archive().expect("failed to convert dar to owned archive")
        })
    }
}
