//! A minimal subset of `OpenAPI` data types required to encode a Daml Dar.
use std::collections::BTreeMap;

use serde::Serialize;

use crate::schema::Schema;

const OPEN_API_VERSION: &str = "3.1.0";
const OPEN_API_SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

#[derive(Debug, Serialize)]
pub struct OpenAPI {
    pub openapi: String,
    pub info: Info,
    #[serde(rename = "jsonSchemaDialect")]
    pub json_schema_dialect: Option<String>,
    pub servers: Vec<Server>,
    pub paths: Paths,
    pub components: Option<Components>,
    pub tags: Vec<Tag>,
}

impl OpenAPI {
    pub fn new(info: Info, servers: Vec<Server>, paths: Paths, components: Components, tags: Vec<Tag>) -> Self {
        Self {
            openapi: OPEN_API_VERSION.to_string(),
            info,
            json_schema_dialect: Some(OPEN_API_SCHEMA_DIALECT.to_string()),
            servers,
            paths,
            components: Some(components),
            tags,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Info {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub version: String,
}

impl Info {
    pub fn new(
        title: String,
        summary: Option<String>,
        contact: Option<Contact>,
        description: Option<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            title,
            summary,
            contact,
            description,
            version: version.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Server {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Server {
    pub const fn new(url: String, description: Option<String>) -> Self {
        Self {
            url,
            description,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Tag {
    pub const fn new(name: String, description: Option<String>) -> Self {
        Self {
            name,
            description,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

impl Contact {
    pub const fn new(name: Option<String>, url: Option<String>, email: Option<String>) -> Self {
        Self {
            name,
            url,
            email,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Paths {
    #[serde(flatten)]
    pub items: BTreeMap<String, PathItem>,
}

impl Paths {
    pub fn new(items: BTreeMap<String, PathItem>) -> Self {
        Self {
            items,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Components {
    pub schemas: BTreeMap<String, Schema>,
}

impl Components {
    pub fn new(schemas: BTreeMap<String, Schema>) -> Self {
        Self {
            schemas,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
}

impl PathItem {
    pub const fn new(post: Option<Operation>) -> Self {
        Self {
            post,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "operationId")]
    pub operation_id: Option<String>,
    pub tags: Vec<String>,
    #[serde(rename = "requestBody")]
    pub request_body: RequestBody,
    pub responses: Responses,
}

impl Operation {
    pub const fn new(
        description: Option<String>,
        operation_id: Option<String>,
        tags: Vec<String>,
        request_body: RequestBody,
        responses: Responses,
    ) -> Self {
        Self {
            description,
            operation_id,
            tags,
            request_body,
            responses,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub content: BTreeMap<String, MediaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ResponseType {
    Response(Response),
}

#[derive(Debug, Serialize)]
pub struct Responses {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<ResponseType>,
    #[serde(flatten)]
    pub responses: BTreeMap<String, ResponseType>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub description: String,
    pub content: BTreeMap<String, MediaType>,
}

#[derive(Debug, Serialize)]
pub struct Reference {
    #[serde(rename = "$ref")]
    pub dollar_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MediaType {
    pub schema: Schema,
}

impl MediaType {
    pub const fn new(schema: Schema) -> Self {
        Self {
            schema,
        }
    }
}
