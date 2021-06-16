use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaUnit<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: &'static str,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: bool,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaBool<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: &'static str,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaText<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: &'static str,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaParty<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: &'static str,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaContractId<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: &'static str,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaDate<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: &'static str,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaTimestamp<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: &'static str,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaInt64<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: Value,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaDecimal<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: Value,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaList<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: &'static str,
    pub items: Value,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaTextMap<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: &'static str,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: Value,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaGenMap<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: &'static str,
    pub items: DamlJsonSchemaGenMapItems,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaGenMapItems {
    #[serde(rename = "type")]
    pub ty: &'static str,
    pub items: [Value; 2],
    #[serde(rename = "minItems")]
    pub min_items: usize,
    #[serde(rename = "maxItems")]
    pub max_items: usize,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum DamlJsonSchemaOptional<'a> {
    TopLevel(DamlJsonSchemaOptionalTopLevel<'a>),
    NonTopLevel(DamlJsonSchemaOptionalNonTopLevel<'a>),
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaOptionalTopLevel<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "oneOf")]
    pub one_of: [Value; 2],
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaOptionalNonTopLevel<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "oneOf")]
    pub one_of: [DamlJsonSchemaOptionalNonTopLevelOneOf; 2],
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaOptionalNonTopLevelOneOf {
    #[serde(rename = "type")]
    pub ty: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Value>,
    #[serde(rename = "minItems")]
    pub min_items: usize,
    #[serde(rename = "maxItems")]
    pub max_items: usize,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaRecord<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "oneOf")]
    pub one_of: [Value; 2],
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaRecordAsObject<'a> {
    #[serde(rename = "type")]
    pub ty: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    pub properties: BTreeMap<&'a str, Value>,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: bool,
    pub required: Vec<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaRecordAsArray<'a> {
    #[serde(rename = "type")]
    pub ty: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    pub items: Vec<Value>,
    #[serde(rename = "minItems")]
    pub min_items: usize,
    #[serde(rename = "maxItems")]
    pub max_items: usize,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaVariant<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "oneOf")]
    pub one_of: Vec<Value>,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaVariantArm<'a> {
    #[serde(rename = "type")]
    pub ty: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    pub properties: Value,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: bool,
    pub required: Vec<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct DamlJsonSchemaEnum<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(rename = "type")]
    pub ty: &'static str,
    #[serde(rename = "enum")]
    pub data_enum: Vec<&'a str>,
}
