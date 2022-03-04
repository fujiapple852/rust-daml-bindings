use std::collections::HashMap;
use std::convert::TryFrom;

use daml::json_api::error::{DamlJsonReqConError, DamlJsonReqConResult};
use daml::json_api::request_converter::DamlJsonTemplateId;
use serde::Deserialize;

///
#[derive(Debug, Deserialize, Default)]
#[serde(transparent)]
pub struct TemplateFilterInput {
    pub items: HashMap<String, ChoiceFilter>,
}

/// A collection of template and choice filtering rules.
#[derive(Debug, Default)]
pub struct TemplateFilter {
    pub items: HashMap<DamlJsonTemplateId, ChoiceFilter>,
}

impl TryFrom<TemplateFilterInput> for TemplateFilter {
    type Error = DamlJsonReqConError;

    fn try_from(value: TemplateFilterInput) -> Result<Self, Self::Error> {
        let items = value
            .items
            .into_iter()
            .map(|(template, filter)| DamlJsonTemplateId::try_from(template.as_str()).map(|di| (di, filter)))
            .collect::<DamlJsonReqConResult<HashMap<_, _>>>()?;
        Ok(Self {
            items,
        })
    }
}

/// Specifies Which choices of a given template to include.
#[derive(Debug, Deserialize)]
pub enum ChoiceFilter {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "all")]
    All,
    #[serde(rename = "selected")]
    Selected(Vec<String>),
}
