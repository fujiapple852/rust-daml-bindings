use itertools::{chain, zip};
use serde_json::Value;

use crate::common::DataId;
use crate::format::{format_daml_template, format_oas_template, format_path_regex_safe};

/// Make Daml JSON API schema items.
pub struct DamlJsonApiSchema {
    reference_prefix: String,
    emit_package_id: bool,
}

impl DamlJsonApiSchema {
    pub fn new(reference_prefix: impl Into<String>, emit_package_id: bool) -> Self {
        Self {
            reference_prefix: reference_prefix.into(),
            emit_package_id,
        }
    }

    /// Make a JSON schema value which represents a Daml JSON API error response.
    ///
    /// The Daml JSON API may only produce four error code (400, 401, 404 & 500) and in each case the schema of the
    /// error is identical and so the below JSON schema object is used to represent all of these.
    pub fn make_error_response() -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "status": {
                  "enum": [400, 401, 404, 500]
                },
                "errors": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                },
                "warnings": {
                    "type": "object"
                }
              },
              "required": [
                "status",
                "errors"
              ],
              "additionalProperties": false
            }
        )
    }

    /// Make a JSON schema value which represents the `Archive` choice.
    pub fn make_archive_type() -> Value {
        serde_json::json!(
            {
                "type": "object",
                "additionalProperties": false,
            }
        )
    }

    /// Make a JSON schema value which represents a Daml JSON API `create` request for a given `TemplateId`.
    pub fn make_create_request(&self, template_id: &DataId) -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "templateId": self.make_template_pattern(template_id),
                "payload": {
                  "$ref": self.make_schema_ref_url(&format_oas_template(template_id))
                },
                "meta": {
                  "type": "object",
                  "properties": {
                    "commandId": {
                      "type": "string"
                    }
                  },
                  "required": ["commandId"]
                }
              },
              "required": [
                "templateId",
                "payload"
              ],
              "additionalProperties": false
            }
        )
    }

    /// Make a JSON schema value which represents a Daml JSON API `create` response for a given `TemplateId`.
    pub fn make_create_response(&self, template_id: &DataId) -> Value {
        Self::make_success_response(&self.make_create_event(template_id))
    }

    /// DOCME
    pub fn make_create_and_exercise_request(&self, template_id: &DataId, choice_id: &str, args: &Value) -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "templateId": self.make_template_pattern(template_id),
                "payload": {
                  "$ref": self.make_schema_ref_url(&format_oas_template(template_id))
                },
                "choice": Self::make_choice(choice_id),
                "argument": args
              },
              "required": [
                "templateId",
                "payload",
                "choice",
                "argument"
              ],
              "additionalProperties": false
            }
        )
    }

    /// DOCME
    pub fn make_exercise_by_id_request(&self, template_id: &DataId, choice_id: &str, args: &Value) -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "templateId": self.make_template_pattern(template_id),
                "contractId": {
                  "type": "string"
                },
                "choice": Self::make_choice(choice_id),
                "argument": args,
              },
              "required": [
                "templateId",
                "contractId",
                "choice",
                "argument"
              ],
              "additionalProperties": false
            }
        )
    }

    /// DOCME
    pub fn make_exercise_by_key_request(
        &self,
        template_id: &DataId,
        choice_id: &str,
        args: &Value,
        key: &Value,
    ) -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "templateId": self.make_template_pattern(template_id),
                "key": key,
                "choice": Self::make_choice(choice_id),
                "argument": args
              },
              "required": [
                "templateId",
                "key",
                "choice",
                "argument"
              ],
              "additionalProperties": false
            }
        )
    }

    /// DOCME
    pub fn make_exercise_response(&self, return_type: &Value, created: &[DataId], archived: &[DataId]) -> Value {
        Self::make_success_response(&serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "exerciseResult": return_type,
                    "events": {
                        "type": "array",
                        "items": {
                            "oneOf": self.make_create_and_archive_events(created, archived)
                        }
                    },
                },
                "required": ["exerciseResult", "events"],
                "additionalProperties": false
            }
        ))
    }

    /// DOCME
    pub fn make_fetch_by_id_request() -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "contractId": {
                  "type": "string"
                },
              },
              "required": [
                "contractId"
              ],
              "additionalProperties": false
            }
        )
    }

    /// DOCME
    pub fn make_fetch_by_key_request(&self, template_id: &DataId, key: &Value) -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "templateId": self.make_template_pattern(template_id),
                "key": key,
              },
              "required": [
                "templateId", "key"
              ],
              "additionalProperties": false
            }
        )
    }

    /// DOCME
    pub fn make_fetch_response(&self, template_id: &DataId) -> Value {
        Self::make_success_response(&Self::make_optional_result(&self.make_create_event(template_id)))
    }

    /// Make a Contracts Query Stream request (single).
    ///
    /// See [Contracts Query Stream](https://docs.daml.com/json-api/index.html#contracts-query-stream)
    pub fn make_stream_query_single_request(&self, templates: &[DataId]) -> Value {
        self.make_query_request(templates)
    }

    /// Make a Contracts Query Stream request (multi).
    ///
    /// See [Contracts Query Stream](https://docs.daml.com/json-api/index.html#contracts-query-stream)
    pub fn make_stream_query_multi_request(&self, templates: &[DataId]) -> Value {
        serde_json::json!(
            {
                "type": "array",
                "items": self.make_query_request(templates),
                "minItems": 1
            }
        )
    }

    /// Make a Contracts Query Stream request (offset).
    ///
    /// See [Contracts Query Stream](https://docs.daml.com/json-api/index.html#contracts-query-stream)
    pub fn make_stream_query_offset_request() -> Value {
        serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "offset": {
                        "type": "integer"
                    }
                },
                "additionalProperties": false
            }
        )
    }

    /// Make Fetch by Key Contracts Stream request.
    ///
    /// See [Fetch by Key Contracts Stream](https://docs.daml.com/json-api/index.html#fetch-by-key-contracts-stream)
    pub fn make_stream_fetch_request(&self, templates: &[DataId], keys: &[Value]) -> Value {
        let template_fetch_items =
            zip(templates, keys).into_iter().map(|(t, k)| self.make_fetch_item(t, k)).collect::<Vec<_>>();
        serde_json::json!(
            {
                "type": "array",
                "description": "The application/json body that must be sent first, formatted according to the following rule",
                "items": {
                    "oneOf": template_fetch_items
                },
                "minItems": 1
            }
        )
    }

    /// Make Fetch & Query Stream events.
    pub fn make_stream_events_response(&self, created: &[DataId], archived: &[DataId]) -> Value {
        // TODO this is technically wrong as stream events don't contain warnings
        // TODO is offset mandatory?
        Self::make_success_response(&serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "events": {
                        "type": "array",
                        "items": {
                            "oneOf": self.make_create_and_archive_stream_events(created, archived)
                        }
                    },
                    "offset": {
                        "type": "string"
                    }
                },
                "required": ["events", "offset"],
                "additionalProperties": false
            }
        ))
    }

    /// Make a warning response suitable for use by the streaming endpoints.
    pub fn make_stream_warnings() -> Value {
        serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "unknownTemplateIds": {
                        "type": "array",
                        "description": "template ID strings",
                        "items": {
                            "type": "string"
                        }
                    }
                },
                "required": ["unknownTemplateIds"],
                "additionalProperties": false
            }
        )
    }

    /// Make an error response suitable for the streaming API endpoints.
    ///
    /// Note: does not contain any optional warnings.
    pub fn make_stream_errors() -> Value {
        serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "errors": {
                        "type": "array",
                        "description": "error messages",
                        "items": {
                            "type": "string"
                        }
                    },
                    "status": {
                        "enum": [400, 401, 404, 500]
                    }
                },
                "required": ["errors", "status"],
                "additionalProperties": false
            }
        )
    }

    fn make_create_and_archive_stream_events(
        &self,
        create_template_ids: &[DataId],
        archived_template_ids: &[DataId],
    ) -> Value {
        let active = create_template_ids.iter().map(|create| self.make_stream_created(create));
        let archived = archived_template_ids.iter().map(|archive| self.make_stream_archived(archive));
        chain(active, archived).collect()
    }

    fn make_stream_archived(&self, template_id: &DataId) -> Value {
        serde_json::json!(
            {
                "type": "object",
                "description": format!("archived: {}", format_daml_template(template_id)),
                "properties": {
                    "archived": self.make_archive_event(template_id),
                },
                "required": ["archived"],
                "additionalProperties": false
            }
        )
    }

    fn make_stream_created(&self, template_id: &DataId) -> Value {
        serde_json::json!(
            {
                "type": "object",
                "description": format!("created: {}", format_daml_template(template_id)),
                "properties": {
                    "created": self.make_create_event(template_id),
                    "matchedQueries": {
                        "type": "array",
                        "items": {
                            "type": "integer"
                        }
                    }
                },
                "required": ["created", "matchedQueries"],
                "additionalProperties": false
            }
        )
    }

    fn make_fetch_item(&self, template_id: &DataId, key: &Value) -> Value {
        serde_json::json!(
            {
                "type": "object",
                "description": format_daml_template(template_id),
                "properties": {
                    "templateId": self.make_template_pattern(template_id),
                    "key": key,
                },
                "required": ["templateId", "key"],
                "additionalProperties": false
            }
        )
    }

    fn make_query_request(&self, templates: &[DataId]) -> Value {
        let template_patterns = templates.iter().map(|t| self.make_template_pattern(t)).collect::<Vec<_>>();
        serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "templateIds": {
                        "type": "array",
                        "items": {
                            "oneOf": template_patterns
                        },
                        "minItems": 1
                    },
                    "query": {
                        "type": "object",
                        "externalDocs": {
                            "description": "See the Daml documentation for details of the query language",
                            "url": "https://docs.daml.com/json-api/search-query-language.html"
                        }
                    }
                },
                "required": ["templateIds"],
                "additionalProperties": false
            }
        )
    }

    /// Make a JSON schema value which represents a `$ref` reference to another schema item.
    pub fn make_schema_ref(&self, dollar_ref: &str) -> Value {
        serde_json::json!({ "$ref": self.make_schema_ref_url(dollar_ref) })
    }

    fn make_success_response(result: &Value) -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "status": {
                  "type": "integer",
                  "const": 200
                },
                "result": result,
                "warnings": {
                    "type": "object"
                }
              },
              "required": [
                "status",
                "result"
              ],
              "additionalProperties": false
            }
        )
    }

    fn make_create_and_archive_events(
        &self,
        create_template_ids: &[DataId],
        archived_template_ids: &[DataId],
    ) -> Value {
        let active = create_template_ids.iter().map(|create| self.make_created(create));
        let archived = archived_template_ids.iter().map(|archive| self.make_archived(archive));
        chain(active, archived).collect()
    }

    fn make_archived(&self, template_id: &DataId) -> Value {
        serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "archived": self.make_archive_event(template_id),
                },
                "required": ["archived"],
                "additionalProperties": false
            }
        )
    }

    fn make_created(&self, template_id: &DataId) -> Value {
        serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "created": self.make_create_event(template_id),
                },
                "required": ["created"],
                "additionalProperties": false
            }
        )
    }

    fn make_archive_event(&self, template_id: &DataId) -> Value {
        serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "contractId": {
                        "type": "string"
                    },
                    "templateId": self.make_template_pattern(template_id)
                },
                "required": ["contractId", "templateId"],
                "additionalProperties": false
            }
        )
    }

    fn make_create_event(&self, template_id: &DataId) -> Value {
        serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "observers": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        }
                    },
                    "agreementText": {
                         "type": "string"
                    },
                    "payload": {
                        "$ref": self.make_schema_ref_url(&format_oas_template(template_id))
                    },
                    "signatories": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        }
                    },
                    "contractId": {
                         "type": "string"
                    },
                    "templateId": self.make_template_pattern(template_id)
                },
                "required": ["observers", "agreementText", "payload", "signatories", "contractId", "templateId"],
                "additionalProperties": false
            }
        )
    }

    fn make_template_pattern(&self, template_id: &DataId) -> Value {
        let path = format_path_regex_safe(&template_id.module_path);
        let pattern = if self.emit_package_id {
            format!("^({}:)?{}:{}$", template_id.package_id, path, &template_id.name)
        } else {
            format!("^(.+:)?{}:{}$", path, &template_id.name)
        };
        serde_json::json!(
            {
                "type": "string",
                "pattern": pattern
            }
        )
    }

    fn make_choice(choice_id: &str) -> Value {
        serde_json::json!({ "const": choice_id })
    }

    fn make_optional_result(result: &Value) -> Value {
        serde_json::json!(
            {
              "oneOf": [ { "type": "null" }, result ]
            }
        )
    }

    fn make_schema_ref_url(&self, schema_ref: &str) -> String {
        format!("{}{}", self.reference_prefix, schema_ref)
    }
}
