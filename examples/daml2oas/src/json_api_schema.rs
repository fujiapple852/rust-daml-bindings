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

    // Rest request & response
    //

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
                  "enum": [400, 401, 404, 500],
                  "description": "the [error status code](https://docs.daml.com/json-api/index.html#failure-http-status-400-401-404-500) returned in the HTTP header"
                },
                "errors": {
                    "type": "array",
                    "description": "a JSON array of strings, each string represents one error",
                    "items": {
                        "type": "string"
                    }
                },
                "warnings": {
                    "type": "object",
                    "description": "an optional field with a JSON object, representing one or many warnings",
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
        let payload = serde_json::json!({"$ref": self.make_schema_ref_url(&format_oas_template(template_id))});
        let template_id = self.make_template_pattern_schema(template_id);
        Self::make_create_request_schema(&template_id, &payload)
    }

    /// Make a JSON schema value which represents a Daml JSON API `create` response for a given `TemplateId`.
    pub fn make_create_response(&self, template_id: &DataId) -> Value {
        Self::make_success_response_schema(&self.make_create_event(template_id))
    }

    /// DOCME
    pub fn make_create_and_exercise_request(&self, template_id: &DataId, choice_id: &str, args: &Value) -> Value {
        let payload = serde_json::json!({"$ref": self.make_schema_ref_url(&format_oas_template(template_id))});
        let template_id = self.make_template_pattern_schema(template_id);
        let choice = Self::make_choice_schema(choice_id);
        Self::make_create_and_exercise_request_schema(&template_id, &payload, &choice, args)
    }

    /// DOCME
    pub fn make_exercise_by_id_request(&self, template_id: &DataId, choice_id: &str, args: &Value) -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "templateId": self.make_template_pattern_schema(template_id),
                "contractId": {
                  "type": "string",
                  "description": "the id of the contract on which to exercise the choice"
                },
                "choice": Self::make_choice_schema(choice_id),
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
                "templateId": self.make_template_pattern_schema(template_id),
                "key": key,
                "choice": Self::make_choice_schema(choice_id),
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
        let event_items = serde_json::json!(
            {
                "oneOf": self.make_create_and_archive_events(created, archived)
            }
        );
        Self::make_exercise_response_schema(return_type, &event_items)
    }

    /// DOCME
    pub fn make_fetch_by_id_request() -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "contractId": {
                  "type": "string",
                  "description": "the id of the contract to fetch"
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
                "templateId": self.make_template_pattern_schema(template_id),
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
        Self::make_success_response_schema(&Self::make_optional_result_schema(&self.make_create_event(template_id)))
    }

    // Rest request & response (general)
    //

    /// Make a JSON schema value which represents a Daml JSON API `create` request for general template.
    pub fn make_general_create_request() -> Value {
        let payload = Self::make_general_payload_schema();
        let template_id = Self::make_general_template_id_schema();
        Self::make_create_request_schema(&template_id, &payload)
    }

    /// Make a JSON schema value which represents a Daml JSON API `create` response for an general template.
    pub fn make_general_create_response() -> Value {
        Self::make_success_response_schema(&Self::make_general_create_event())
    }

    /// DOCME
    pub fn make_general_create_and_exercise_request() -> Value {
        let payload = Self::make_general_payload_schema();
        let template_id = Self::make_general_template_id_schema();
        let choice = Self::make_general_choice_schema();
        let args = Self::make_general_args_schema();
        Self::make_create_and_exercise_request_schema(&template_id, &payload, &choice, &args)
    }

    /// DOCME
    pub fn make_general_exercise_request() -> Value {
        serde_json::json!({
            "oneOf": [Self::make_general_exercise_by_id_request_schema(), Self::make_general_exercise_by_key_request_schema()]
        })
    }

    /// DOCME
    pub fn make_general_exercise_response() -> Value {
        let return_type = Self::make_general_return_type_schema();
        let event_items = serde_json::json!({ "oneOf": Self::make_general_create_and_archive_events() });
        Self::make_exercise_response_schema(&return_type, &event_items)
    }

    /// DOCME
    pub fn make_general_fetch_request() -> Value {
        serde_json::json!({
            "oneOf": [Self::make_general_fetch_by_id_request_schema(), Self::make_general_fetch_by_key_request_schema()]
        })
    }

    /// DOCME
    pub fn make_general_fetch_response() -> Value {
        Self::make_success_response_schema(&Self::make_optional_result_schema(&Self::make_general_create_event()))
    }

    // Stream request & response
    //

    /// Make a Contracts Query Stream request (single).
    ///
    /// See [Contracts Query Stream](https://docs.daml.com/json-api/index.html#contracts-query-stream)
    pub fn make_stream_query_single_request(&self, templates: &[DataId]) -> Value {
        self.make_query_request_schema(templates)
    }

    /// Make a Contracts Query Stream request (multi).
    ///
    /// See [Contracts Query Stream](https://docs.daml.com/json-api/index.html#contracts-query-stream)
    pub fn make_stream_query_multi_request(&self, templates: &[DataId]) -> Value {
        serde_json::json!(
            {
                "type": "array",
                "items": self.make_query_request_schema(templates),
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
            zip(templates, keys).into_iter().map(|(t, k)| self.make_fetch_item_schema(t, k)).collect::<Vec<_>>();
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
        Self::make_success_response_schema(&serde_json::json!(
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

    // Implementations (template specific)
    //

    fn make_create_request_schema(template_id: &Value, payload: &Value) -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "templateId": template_id,
                "payload": payload,
                "meta": {
                  "type": "object",
                  "properties": {
                    "commandId": {
                      "type": "string",
                      "description": "the commandId used when submitting a command to the ledger"
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

    fn make_create_and_exercise_request_schema(
        template_id: &Value,
        payload: &Value,
        choice: &Value,
        args: &Value,
    ) -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "templateId": template_id,
                "payload": payload,
                "choice": choice,
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

    fn make_exercise_response_schema(return_type: &Value, event_items: &Value) -> Value {
        Self::make_success_response_schema(&serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "exerciseResult": return_type,
                    "events": {
                        "type": "array",
                        "items": event_items
                    },
                },
                "required": ["exerciseResult", "events"],
                "additionalProperties": false
            }
        ))
    }

    fn make_create_and_archive_stream_events(
        &self,
        create_template_ids: &[DataId],
        archived_template_ids: &[DataId],
    ) -> Value {
        let active = create_template_ids.iter().map(|create| self.make_stream_created_schema(create));
        let archived = archived_template_ids.iter().map(|archive| self.make_stream_archived_schema(archive));
        chain(active, archived).collect()
    }

    fn make_stream_archived_schema(&self, template_id: &DataId) -> Value {
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

    fn make_stream_created_schema(&self, template_id: &DataId) -> Value {
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

    fn make_fetch_item_schema(&self, template_id: &DataId, key: &Value) -> Value {
        serde_json::json!(
            {
                "type": "object",
                "description": format_daml_template(template_id),
                "properties": {
                    "templateId": self.make_template_pattern_schema(template_id),
                    "key": key,
                },
                "required": ["templateId", "key"],
                "additionalProperties": false
            }
        )
    }

    fn make_query_request_schema(&self, templates: &[DataId]) -> Value {
        let template_patterns = templates.iter().map(|t| self.make_template_pattern_schema(t)).collect::<Vec<_>>();
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

    fn make_success_response_schema(result: &Value) -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "status": {
                  "type": "integer",
                  "description": "field matches the HTTP response status code returned in the HTTP header",
                  "const": 200
                },
                "result": result,
                "warnings": {
                    "type": "object",
                    "description": "an optional field with a JSON object, representing one or many warnings"
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
        let created = create_template_ids.iter().map(|create| self.make_created_schema(create));
        let archived = archived_template_ids.iter().map(|archive| self.make_archived_schema(archive));
        chain(created, archived).collect()
    }

    fn make_created_schema(&self, template_id: &DataId) -> Value {
        serde_json::json!(
            {
                "type": "object",
                "description": format!("created event ({})", format_daml_template(template_id)),
                "properties": {
                    "created": self.make_create_event(template_id),
                },
                "required": ["created"],
                "additionalProperties": false
            }
        )
    }

    fn make_archived_schema(&self, template_id: &DataId) -> Value {
        serde_json::json!(
            {
                "type": "object",
                "description": format!("archived event ({})", format_daml_template(template_id)),
                "properties": {
                    "archived": self.make_archive_event(template_id),
                },
                "required": ["archived"],
                "additionalProperties": false
            }
        )
    }

    fn make_archive_event(&self, template_id: &DataId) -> Value {
        let template_id = self.make_template_pattern_schema(template_id);
        Self::make_archive_event_schema(&template_id)
    }

    fn make_create_event(&self, template_id: &DataId) -> Value {
        let payload = serde_json::json!({"$ref": self.make_schema_ref_url(&format_oas_template(template_id))});
        let template_id = self.make_template_pattern_schema(template_id);
        Self::make_create_event_schema(&template_id, &payload)
    }

    // Implementations (general)
    //

    fn make_general_exercise_by_id_request_schema() -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "templateId": Self::make_general_template_id_schema(),
                "contractId": {
                  "type": "string",
                  "description": "the id of the contract on which to exercise the choice"
                },
                "choice": Self::make_general_choice_schema(),
                "argument": Self::make_general_args_schema(),
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

    fn make_general_exercise_by_key_request_schema() -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "templateId": Self::make_general_template_id_schema(),
                "key": Self::make_general_key_schema(),
                "choice": Self::make_general_choice_schema(),
                "argument": Self::make_general_args_schema(),
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

    fn make_general_create_and_archive_events() -> Value {
        serde_json::json!([Self::make_general_created_schema(), Self::make_general_archived_schema()])
    }

    fn make_general_fetch_by_id_request_schema() -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "contractId": {
                  "type": "string",
                  "description": "the id of the contract to fetch"
                },
              },
              "required": [
                "contractId"
              ],
              "additionalProperties": false
            }
        )
    }

    fn make_general_fetch_by_key_request_schema() -> Value {
        serde_json::json!(
            {
              "type": "object",
              "properties": {
                "templateId": Self::make_general_template_id_schema(),
                "key": Self::make_general_key_schema(),
              },
              "required": [
                "templateId", "key"
              ],
              "additionalProperties": false
            }
        )
    }

    fn make_general_created_schema() -> Value {
        serde_json::json!(
            {
                "type": "object",
                "description": "created event",
                "properties": {
                    "created": &Self::make_general_create_event(),
                },
                "required": ["created"],
                "additionalProperties": false
            }
        )
    }

    fn make_general_archived_schema() -> Value {
        serde_json::json!(
            {
                "type": "object",
                "description": "archived event",
                "properties": {
                    "archived": Self::make_general_archive_event(),
                },
                "required": ["archived"],
                "additionalProperties": false
            }
        )
    }

    fn make_general_create_event() -> Value {
        let payload = Self::make_general_payload_schema();
        let template_id = Self::make_general_template_id_schema();
        Self::make_create_event_schema(&template_id, &payload)
    }

    fn make_general_archive_event() -> Value {
        let template_id = Self::make_general_template_id_schema();
        Self::make_archive_event_schema(&template_id)
    }

    fn make_general_payload_schema() -> Value {
        serde_json::json!(
            {
                "type": "object",
                "description": "contract fields as defined in the Daml template and formatted according to [Daml-LF JSON Encoding](https://docs.daml.com/json-api/lf-value-specification.html)"
            }
        )
    }

    fn make_general_template_id_schema() -> Value {
        serde_json::json!(
            {
                "type": "string",
                "description": "templateId is the contract template identifier, which can be formatted as either `<package ID>:<module>:<entity>` or `<module>:<entity>` if contract template can be uniquely identified by its module and entity name"
            }
        )
    }

    fn make_general_args_schema() -> Value {
        serde_json::json!(
            {
                "type": "object",
                "description": "contract choice argument(s) formatted according to [Daml-LF JSON Encoding](https://docs.daml.com/json-api/lf-value-specification.html)"
            }
        )
    }

    fn make_general_choice_schema() -> Value {
        serde_json::json!(
            {
                "type": "string",
                "description": "Daml contract choice, that is being exercised"
            }
        )
    }

    fn make_general_key_schema() -> Value {
        serde_json::json!(
            {
                "type": "object",
                "description": "contract key, formatted according to the [Daml-LF JSON Encoding](https://docs.daml.com/json-api/lf-value-specification.html)"
            }
        )
    }

    fn make_general_return_type_schema() -> Value {
        serde_json::json!(
            {
                "type": "object",
                "description": "field contains the return value of the exercised contract choice, formatted according to [Daml-LF JSON Encoding](https://docs.daml.com/json-api/lf-value-specification.html)"
            }
        )
    }

    fn make_create_event_schema(template_id: &Value, payload: &Value) -> Value {
        serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "observers": {
                        "type": "array",
                        "description": "the list of observers of the contract",
                        "items": {
                            "type": "string"
                        }
                    },
                    "agreementText": {
                         "type": "string",
                         "description": "the agreement text of the contract"
                    },
                    "payload": payload,
                    "signatories": {
                        "type": "array",
                        "description": "the list of parties who are signatories of the contract",
                        "items": {
                            "type": "string"
                        }
                    },
                    "contractId": {
                         "type": "string",
                         "description": "field contains created contract details"
                    },
                    "templateId": template_id
                },
                "required": ["observers", "agreementText", "payload", "signatories", "contractId", "templateId"],
                "additionalProperties": false
            }
        )
    }

    fn make_archive_event_schema(template_id: &Value) -> Value {
        serde_json::json!(
            {
                "type": "object",
                "properties": {
                    "contractId": {
                        "type": "string",
                        "description": "field contains created contract details"
                    },
                    "templateId": template_id
                },
                "required": ["contractId", "templateId"],
                "additionalProperties": false
            }
        )
    }

    fn make_template_pattern_schema(&self, template_id: &DataId) -> Value {
        let path = format_path_regex_safe(&template_id.module);
        let pattern = if self.emit_package_id {
            format!("^({}:)?{}:{}$", template_id.package_id.as_deref().unwrap_or_default(), path, &template_id.entity)
        } else {
            format!("^(.+:)?{}:{}$", path, &template_id.entity)
        };
        serde_json::json!(
            {
                "type": "string",
                "description": format!("templateId is the contract template identifier for {}", format_daml_template(template_id)),
                "pattern": pattern
            }
        )
    }

    fn make_choice_schema(choice_id: &str) -> Value {
        serde_json::json!({ "const": choice_id })
    }

    fn make_optional_result_schema(result: &Value) -> Value {
        serde_json::json!(
            {
              "oneOf": [ result, { "type": "null", "description": "Contract not found" } ]
            }
        )
    }

    fn make_schema_ref_url(&self, schema_ref: &str) -> String {
        format!("{}{}", self.reference_prefix, schema_ref)
    }
}
