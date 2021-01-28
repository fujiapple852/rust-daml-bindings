use crate::data::{DamlJsonCreatedEvent, DamlJsonExerciseResult, DamlJsonParty};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::iter::once;

/// DAML JSON API request metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonRequestMeta {
    #[serde(rename = "commandId")]
    pub command_id: String,
}

impl DamlJsonRequestMeta {
    pub fn new(command_id: impl Into<String>) -> Self {
        Self {
            command_id: command_id.into(),
        }
    }
}

/// DAML JSON API create contract request.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonCreateRequest {
    #[serde(rename = "templateId")]
    pub template_id: String,
    pub payload: Value,
    pub meta: Option<DamlJsonRequestMeta>,
}

impl DamlJsonCreateRequest {
    /// Create a new `DamlJsonCreateRequest` for a given template id and contract payload.
    pub fn new(template_id: impl Into<String>, payload: Value) -> Self {
        Self {
            template_id: template_id.into(),
            payload,
            meta: None,
        }
    }

    /// Create a new `DamlJsonCreateRequest` with metadata for a given template id and contract payload.
    pub fn new_with_meta(template_id: impl Into<String>, payload: Value, meta: DamlJsonRequestMeta) -> Self {
        Self {
            template_id: template_id.into(),
            payload,
            meta: Some(meta),
        }
    }
}

/// DAML JSON API create contract response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonCreateResponse {
    pub status: u16,
    pub result: DamlJsonCreatedEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<HashMap<String, Vec<String>>>,
}

/// Represents either a DAML JSON API [`DamlJsonExerciseRequest`] or [`DamlJsonExerciseByKeyRequest`].
///
/// This is required as the DAML JSON API uses the same path (`exercise`) for both request types and the only way
/// to uniquely identify which case we have been provided is by checking for the `contractId` and `key` fields.
///
/// To avoid having to first convert to a generic JSON `Value` to decide check which structure has been provided we
/// use the `serde` [untagged enum](https://serde.rs/enum-representations.html#untagged) feature and add an
/// [`DamlJsonInvalidExerciseRequest`] variant which has both `contractId` and `key` fields.  This allow the downstream
/// hander to rejected the request with a suitable error message.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DamlJsonExerciseRequestType {
    Invalid(DamlJsonInvalidExerciseRequest),
    Exercise(DamlJsonExerciseRequest),
    ExerciseByKey(DamlJsonExerciseByKeyRequest),
}

/// DAML JSON API exercise choice request.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonExerciseRequest {
    #[serde(rename = "templateId")]
    pub template_id: String,
    #[serde(rename = "contractId")]
    pub contract_id: String,
    pub choice: String,
    pub argument: Value,
}

impl DamlJsonExerciseRequest {
    /// Create a new `DamlJsonExerciseRequest` for a given template id, contract id, choice name and choice arguments.
    pub fn new(
        template_id: impl Into<String>,
        contract_id: impl Into<String>,
        choice: impl Into<String>,
        argument: Value,
    ) -> Self {
        Self {
            template_id: template_id.into(),
            contract_id: contract_id.into(),
            choice: choice.into(),
            argument,
        }
    }
}

/// DAML JSON API exercise choice response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonExerciseResponse {
    pub status: u16,
    pub result: DamlJsonExerciseResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<HashMap<String, Vec<String>>>,
}

/// DAML JSON API exercise choice by key request.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonExerciseByKeyRequest {
    #[serde(rename = "templateId")]
    pub template_id: String,
    pub key: Value,
    pub choice: String,
    pub argument: Value,
}

impl DamlJsonExerciseByKeyRequest {
    /// Create a new `DamlJsonExerciseByKeyRequest` for a given template id, contract key, choice name and choice
    /// arguments.
    pub fn new(template_id: impl Into<String>, key: Value, choice: impl Into<String>, argument: Value) -> Self {
        Self {
            template_id: template_id.into(),
            key,
            choice: choice.into(),
            argument,
        }
    }
}

/// DAML JSON API exercise choice by key response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonExerciseByKeyResponse {
    pub status: u16,
    pub result: DamlJsonExerciseResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<HashMap<String, Vec<String>>>,
}

/// An invalid DAML JSON API exercise choice request (`key` and `contractId` are mutually exclusive).
///
/// See [`DamlJsonExerciseRequestType`] for details.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonInvalidExerciseRequest {
    #[serde(rename = "templateId")]
    pub template_id: String,
    #[serde(rename = "contractId")]
    pub contract_id: String,
    pub key: Value,
    pub choice: String,
    pub argument: Value,
}

/// DAML JSON API create contract and exercise choice request.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonCreateAndExerciseRequest {
    #[serde(rename = "templateId")]
    pub template_id: String,
    pub payload: Value,
    pub choice: String,
    pub argument: Value,
}

impl DamlJsonCreateAndExerciseRequest {
    /// Create a new `DamlJsonCreateAndExerciseRequest` for a given template id, payload, choice name and choice
    /// arguments.
    pub fn new(template_id: impl Into<String>, payload: Value, choice: impl Into<String>, argument: Value) -> Self {
        Self {
            template_id: template_id.into(),
            payload,
            choice: choice.into(),
            argument,
        }
    }
}

/// DAML JSON API create contract and exercise choice response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonCreateAndExerciseResponse {
    pub status: u16,
    pub result: DamlJsonExerciseResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<HashMap<String, Vec<String>>>,
}

/// DAML JSON API fetch contract by id request.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonFetchRequest {
    #[serde(rename = "contractId")]
    pub contract_id: String,
}

impl DamlJsonFetchRequest {
    /// Create a new `DamlJsonFetchRequest` for a given contract id.
    pub fn new(contract_id: impl Into<String>) -> Self {
        Self {
            contract_id: contract_id.into(),
        }
    }
}

/// DAML JSON API fetch contract by id response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonFetchResponse {
    pub status: u16,
    pub result: DamlJsonCreatedEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<HashMap<String, Vec<String>>>,
}

/// DAML JSON API fetch contract by key request.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonFetchByKeyRequest {
    #[serde(rename = "templateId")]
    pub template_id: String,
    pub key: Value,
}

impl DamlJsonFetchByKeyRequest {
    /// Create a new `DamlJsonFetchByKeyRequest` for a given template id and key.
    pub fn new(template_id: impl Into<String>, key: Value) -> Self {
        Self {
            template_id: template_id.into(),
            key,
        }
    }
}

/// DAML JSON API fetch contract by key response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonFetchByKeyResponse {
    pub status: u16,
    pub result: DamlJsonCreatedEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<HashMap<String, Vec<String>>>,
}

/// DAML JSON API query response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonQueryResponse {
    pub status: u16,
    pub result: Vec<DamlJsonCreatedEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<HashMap<String, Vec<String>>>,
}

/// DAML JSON API fetch parties request.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonFetchPartiesRequest(pub Vec<String>);

impl DamlJsonFetchPartiesRequest {
    /// Create a new `DamlJsonFetchPartiesRequest` for a given list of party identifiers.
    pub fn new(identifiers: Vec<String>) -> Self {
        Self(identifiers)
    }
}

/// DAML JSON API fetch parties response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonFetchPartiesResponse {
    pub status: u16,
    pub result: Vec<DamlJsonParty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<HashMap<String, Vec<String>>>,
}

/// DAML JSON API allocate party request.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonAllocatePartyRequest {
    #[serde(rename = "identifierHint")]
    pub identifier_hint: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

impl DamlJsonAllocatePartyRequest {
    /// Create a new `DamlJsonAllocatePartyRequest` with optional identifier hint & display name.
    pub fn new<S: Into<String>>(identifier_hint: Option<S>, display_name: Option<S>) -> Self {
        Self {
            identifier_hint: identifier_hint.map(Into::into),
            display_name: display_name.map(Into::into),
        }
    }
}

/// DAML JSON API allocate party response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonAllocatePartyResponse {
    pub status: u16,
    pub result: DamlJsonParty,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<HashMap<String, Vec<String>>>,
}

/// DAML JSON API list packages response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonListPackagesResponse {
    pub status: u16,
    pub result: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<HashMap<String, Vec<String>>>,
}

/// DAML JSON API upload `Dar` response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonUploadDarResponse {
    pub status: u16,
    pub result: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<HashMap<String, Vec<String>>>,
}

/// DAML JSON API generic error response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonErrorResponse {
    pub status: u16,
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<HashMap<String, Vec<String>>>,
}

impl DamlJsonErrorResponse {
    pub fn single(status: u16, error: String) -> Self {
        Self {
            status,
            errors: vec![error],
            warnings: None,
        }
    }
}

/// Make a warnings map with a single entry.
pub fn make_single_warning(name: impl Into<String>, data: Vec<String>) -> HashMap<String, Vec<String>> {
    once((name.into(), data)).collect::<HashMap<_, _>>()
}
