use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct CompanionData {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub contact: Option<Contact>,
    pub servers: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Contact {
    pub name: Option<String>,
    pub url: Option<String>,
    pub email: Option<String>,
}
