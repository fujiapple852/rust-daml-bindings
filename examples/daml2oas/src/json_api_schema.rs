use crate::common::DataId;
use crate::format::{format_oas_template, format_path_regex_safe};
use itertools::chain;
use serde_json::Value;

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
              ]
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
              ]
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
              ]
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
              ]
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
              ]
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
                "required": ["exerciseResult", "events"]
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
              ]
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
              ]
            }
        )
    }

    /// DOCME
    pub fn make_fetch_response(&self, template_id: &DataId) -> Value {
        Self::make_success_response(&Self::make_optional_result(&self.make_create_event(template_id)))
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
              ]
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
                "required": ["archived"]
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
                "required": ["created"]
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
                "required": ["contractId", "templateId"]
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
                "required": ["observers", "agreementText", "payload", "signatories", "contractId", "templateId"]
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
