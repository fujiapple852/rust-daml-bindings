use serde::{Deserialize, Serialize};
use serde_json::Value;

/// DAML JSON API representation of a ledger event.
#[derive(Debug, Serialize, Deserialize)]
pub enum DamlJsonEvent {
    #[serde(rename = "created")]
    Created(DamlJsonCreatedEvent),
    #[serde(rename = "archived")]
    Archived(DamlJsonArchivedEvent),
}

/// DAML JSON API representation of a ledger contract created event.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonCreatedEvent {
    pub observers: Vec<String>,
    #[serde(rename = "agreementText")]
    pub agreement_text: String,
    pub payload: Value,
    pub signatories: Vec<String>,
    #[serde(rename = "contractId")]
    pub contract_id: String,
    #[serde(rename = "templateId")]
    pub template_id: String,
}

impl DamlJsonCreatedEvent {
    pub fn new(
        observers: Vec<String>,
        agreement_text: String,
        payload: Value,
        signatories: Vec<String>,
        contract_id: String,
        template_id: impl Into<String>,
    ) -> Self {
        Self {
            observers,
            agreement_text,
            payload,
            signatories,
            contract_id,
            template_id: template_id.into(),
        }
    }
}

/// DAML JSON API representation of a ledger contract archived event.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonArchivedEvent {
    #[serde(rename = "contractId")]
    pub contract_id: String,
    #[serde(rename = "templateId")]
    pub template_id: String,
}

impl DamlJsonArchivedEvent {
    pub const fn new(contract_id: String, template_id: String) -> Self {
        Self {
            contract_id,
            template_id,
        }
    }
}

/// DAML JSON API representation of a ledger exercise result.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonExerciseResult {
    #[serde(rename = "exerciseResult")]
    pub exercise_result: Value,
    pub events: Vec<DamlJsonEvent>,
}

/// DAML JSON API representation of a ledger Party.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DamlJsonParty {
    pub identifier: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "isLocal")]
    pub is_local: bool,
}

impl DamlJsonParty {
    pub fn new<S: Into<String>>(identifier: impl Into<String>, display_name: Option<S>, is_local: bool) -> Self {
        Self {
            identifier: identifier.into(),
            display_name: display_name.map(Into::into),
            is_local,
        }
    }
}

/// DAML JSON API representation of a query.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonQuery {
    #[serde(rename = "templateIds")]
    pub template_ids: Vec<String>,
    pub query: Value,
}

impl DamlJsonQuery {
    pub fn new(template_ids: Vec<String>, query: Value) -> Self {
        Self {
            template_ids,
            query,
        }
    }
}
