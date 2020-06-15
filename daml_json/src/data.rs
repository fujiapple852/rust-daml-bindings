use crate::error::{DamlJsonError, DamlJsonResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryFrom;

/// DAML JSON API representation of a ledger event.
#[derive(Debug, Serialize, Deserialize)]
pub enum DamlJsonEvent {
    #[serde(rename = "created")]
    Created(DamlJsonCreatedEvent),
    #[serde(rename = "archived")]
    Archived(DamlJsonExercisedEvent),
}

/// DAML JSON API representation of a ledger created event.
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

/// DAML JSON API representation of a ledger exercised event.
#[derive(Debug, Serialize, Deserialize)]
pub struct DamlJsonExercisedEvent {
    #[serde(rename = "contractId")]
    pub contract_id: String,
    #[serde(rename = "templateId")]
    pub template_id: String,
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

/// A helper representation of a DAML template id.
///
/// This type exists to provide a convenient `TryFrom` impl for converting from a String.
///
/// Template ids are represented as JSON strings of the form `[package_id:]module:entity` in the DAML JSON API and as
/// such this struct does is not required to be `Serialize` or `Deserialize`.
#[derive(Debug)]
pub struct DamlJsonTemplateId {
    pub package_id: Option<String>,
    pub module: Vec<String>,
    pub entity: String,
}

impl DamlJsonTemplateId {
    pub fn new(package_id: Option<String>, module: Vec<String>, entity: String) -> Self {
        Self {
            package_id,
            module,
            entity,
        }
    }
}

impl TryFrom<&str> for DamlJsonTemplateId {
    type Error = DamlJsonError;

    fn try_from(value: &str) -> DamlJsonResult<Self> {
        let splits: Vec<_> = value.split(':').collect();
        match *splits.as_slice() {
            [module, entity] =>
                Ok(Self::new(None, module.split('.').map(ToOwned::to_owned).collect(), entity.to_owned())),
            [package_id, module, entity] => Ok(Self::new(
                Some(package_id.to_owned()),
                module.split('.').map(ToOwned::to_owned).collect(),
                entity.to_owned(),
            )),
            _ => Err(DamlJsonError::TemplateIdFormatError(value.to_owned())),
        }
    }
}
