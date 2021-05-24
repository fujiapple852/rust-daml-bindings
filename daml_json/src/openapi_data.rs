use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct OpenAPI {
    pub openapi: String,
    pub info: Info,
    #[serde(rename = "json_schema_dialect")]
    pub json_schema_dialect: Option<String>,
    pub paths: Paths,
    pub components: Option<Components>,
}

impl OpenAPI {
    pub fn new(info: Info, paths: Paths, components: Components) -> Self {
        Self {
            openapi: "3.1.0".to_string(),
            info,
            json_schema_dialect: Some("https://json-schema.org/draft/2020-12/schema".to_string()),
            paths,
            components: Some(components)
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Info {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub version: String,
}

impl Info {
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            summary: None,
            version: version.into()
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
        Self { items }
    }
}

#[derive(Debug, Serialize)]
pub struct Components {
    pub schemas: BTreeMap<String, Schema>,
}

impl Components {
    pub fn new(schemas: BTreeMap<String, Schema>) -> Self {
        Self { schemas }
    }
}

#[derive(Debug, Serialize)]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
}

// impl PathItem {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

#[derive(Debug, Serialize)]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "operation_id")]
    pub operation_id: String,
    #[serde(rename = "request_body")]
    pub request_body: Reference,
    pub responses: Responses,
    // callbacks: BTreeMap<String, CallbackObject>,
}

// impl Operation {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// #[derive(Debug, Serialize)]
// pub struct RequestBody {
//     description: Option<String>,
//     content: Map<String, MediaType>,
//     required: bool,
// }

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ResponseType {
    Response(Response),
    Reference(Reference)
}

#[derive(Debug, Serialize)]
pub struct Responses {
    pub default: Option<ResponseType>,
    pub responses: BTreeMap<String, ResponseType>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub description: String,
    pub content: BTreeMap<String, MediaType>,
}

// #[derive(Debug, Serialize)]
// pub struct CallbackObject {}

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
    pub schema: Schema
}

#[derive(Debug, Serialize)]
pub struct Schema {

    // TODO this is the actual JSON schema, and so a "$ref" here is a JSON schema ref, not a openapi ref

    // #[serde(rename = "$ref")]
    // dollar_ref: String,

    #[serde(flatten)]
    pub value: Value,

    // discriminator: Discriminator
}

impl Schema {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}

// #[derive(Debug, Serialize)]
// pub struct Discriminator {
//     #[serde(rename = "property_name")]
//     property_name: String,
// }

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use maplit::btreemap;
    use serde_json::json;

    use super::*;

    // For a Dar, for each package/module, we generate one OpenAPI ?  folders for packages/modules, file for template
    // maybe add a dummy parameter to make paths distinct?

    #[test]
    fn test_openapi() -> Result<()> {
        let openapi = OpenAPI {
            openapi: "3.1.0".to_string(),
            info: Info {
                title: "My Dar".to_string(),
                summary: None,
                version: "0.1.0".to_string(),
            },
            json_schema_dialect: Some("https://json-schema.org/draft/2020-12/schema".to_string()),
            paths: Paths {
                items: btreemap! {
                    "/v1/create#DA.PingPong:Ping".to_string() => PathItem {
                        summary: Some("Create a Ping contract".to_string()),
                        description: None,
                        get: None,
                        post: Some(Operation {
                            summary: Some("create Ping".to_string()),
                            description: None,
                            operation_id: "operationId".to_string(),
                            request_body: Reference {
                                dollar_ref: "#/components/schemas/PingRequest".to_string(),
                                description: Some("Ping object".to_string()),
                                summary: None,
                            },
                            responses: Responses {
                                default: Some(ResponseType::Reference(Reference {
                                    dollar_ref: "#/components/schemas/Error".to_string(),
                                    description: Some("Unexpected error".to_string()),
                                   summary: None,
                                })),
                                responses: btreemap! {
                                    "200".to_string() => ResponseType::Response(Response {
                                        description: "success".to_string(),
                                        content: btreemap! {
                                            "application/json".to_string() => MediaType {
                                                schema: Schema {
                                                    value: json!({"$ref": "#/components/schemas/PingResponse"})
                                                }
                                            }
                                        }
                                    })
                                }
                            },
                        })
                    }
                },
            },
            components: Some(Components {
                schemas: btreemap! {
                    "PingRequest".to_string() => Schema {
                        value: json!(
                            {
                                "type": "object",
                                "properties": {
                                    "templateId": {
                                        "type": "string",
                                        "enum": ["PingPong:Ping"]
                                    },
                                    "payload": { "$ref": "#/components/schemas/Ping"}
                                },
                                "requiredProperties": [ "templateId", "payload" ]
                            }
                        )
                    },
                    "PingResponse".to_string() => Schema {
                        value: json!(
                            {
                                "type": "object",
                                "properties": {
                                    "status": { "type": "integer" },
                                    "result": {
                                        "type": "object",
                                        "properties": {
                                            "observers": { "type": null },
                                            "agreementText": { "type": null },
                                            "payload": { "$ref": "#/components/schemas/Ping" },
                                            "signatories": { "type": null },
                                            "contractId": { "type": null },
                                            "templateId": { "type": null },
                                        },
                                        "requiredProperties": [ "observers", "agreementText", "payload", "signatories", "contractId", "templateId" ]
                                    }
                                },
                                "requiredProperties": [ "status", "result" ]
                            }
                        )
                    },
                }
            })
        };
        let expected = json!(
            {
              "openapi": "3.1.0",
              "info": {
                "title": "My Dar",
                "version": "0.1.0"
              },
              "json_schema_dialect": "https://json-schema.org/draft/2020-12/schema",
              "paths": {
                "/v1/create": {
                  "summary": "Create a Ping contract",
                  "post": {
                    "summary": "create Ping",
                    "operation_id": "operationId",
                    "request_body": {
                      "$ref": "#/components/schemas/PingRequest",
                      "description": "Ping object"
                    },
                    "responses": {
                      "default": {
                        "$ref": "#/components/schemas/Error",
                        "description": "Unexpected error"
                      },
                      "responses": {
                        "200": {
                          "description": "success",
                          "content": {
                            "application/json": {
                              "schema": {
                                "$ref": "#/components/schemas/PingResponse"
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              },
              "components": {
                "PingRequest": {
                  "type": "object",
                  "properties": {
                    "templateId": {
                      "type": "string",
                      "enum": [
                        "PingPong:Ping"
                      ]
                    },
                    "payload": {
                      "$ref": "#/components/schemas/Ping"
                    }
                  },
                  "requiredProperties": [
                    "templateId",
                    "payload"
                  ]
                },
                "PingResponse": {
                  "type": "object",
                  "properties": {
                    "status": {
                      "type": "integer"
                    },
                    "result": {
                      "type": "object",
                      "properties": {
                        "observers": {
                          "type": null
                        },
                        "agreementText": {
                          "type": null
                        },
                        "payload": {
                          "$ref": "#/components/schemas/Ping"
                        },
                        "signatories": {
                          "type": null
                        },
                        "contractId": {
                          "type": null
                        },
                        "templateId": {
                          "type": null
                        }
                      },
                      "requiredProperties": [
                        "observers",
                        "agreementText",
                        "payload",
                        "signatories",
                        "contractId",
                        "templateId"
                      ]
                    }
                  },
                  "requiredProperties": [
                    "status",
                    "result"
                  ]
                }
              }
            }
        );
        let actual = serde_json::to_value(openapi)?;
        println!("{}", serde_json::to_string_pretty(&actual)?);
        assert_json_diff::assert_json_eq!(actual, expected);
        Ok(())
    }
}
