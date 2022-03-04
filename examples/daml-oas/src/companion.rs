use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
pub struct CompanionData {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub contact: Option<Contact>,
    pub servers: Option<Vec<String>>,
    pub operations: Option<HashMap<String, OperationInfo>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Contact {
    pub name: Option<String>,
    pub url: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct OperationInfo {
    pub create: Option<String>,
    #[serde(rename = "createAndExercise")]
    pub create_and_exercise: Option<HashMap<String, String>>,
    #[serde(rename = "exerciseById")]
    pub exercise_by_id: Option<HashMap<String, String>>,
    #[serde(rename = "exerciseByKey")]
    pub exercise_by_key: Option<HashMap<String, String>>,
    #[serde(rename = "fetchById")]
    pub fetch_by_id: Option<String>,
    #[serde(rename = "fetchByKey")]
    pub fetch_by_key: Option<String>,
}
