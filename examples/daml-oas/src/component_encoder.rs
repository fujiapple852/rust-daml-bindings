use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

use daml::json_api::schema_encoder::JsonSchemaEncoder;
use daml::lf::element::{DamlArchive, DamlData, DamlModule, DamlPackage};

use crate::common::NamedItem;
use crate::data_searcher::DamlEntitySearcher;
use crate::filter::TemplateFilter;
use crate::format::format_oas_data;
use crate::schema::Schema;
use crate::util::{ChildModulePathOrError, Required};
use itertools::process_results;
use std::convert::identity;
use std::ops::Not;

type NamedSchema = NamedItem<Schema>;

/// Encode Daml data types as JSON schema components.
///
/// This is designed to be used to generate OAS and A2S documents.
pub struct ComponentEncoder<'arc> {
    archive: &'arc DamlArchive<'arc>,
    module_prefix: &'arc [&'arc str],
    json_schema_encoder: &'arc JsonSchemaEncoder<'arc>,
    filter: &'arc TemplateFilter,
}

impl<'arc> ComponentEncoder<'arc> {
    pub const fn new(
        archive: &'arc DamlArchive<'arc>,
        module_prefix: &'arc [&'arc str],
        json_schema_encoder: &'arc JsonSchemaEncoder<'arc>,
        filter: &'arc TemplateFilter,
    ) -> Self {
        Self {
            archive,
            module_prefix,
            json_schema_encoder,
            filter,
        }
    }

    /// Encode the data types in the `DamlArchive` as a map JSON `Schema` objects.
    pub fn encode_schema_components(&self) -> Result<BTreeMap<String, Schema>> {
        Ok(self
            .encode_package(self.archive.main_package().req()?)?
            .into_iter()
            .map(|dt| (dt.name, dt.item))
            .collect::<BTreeMap<String, Schema>>())
    }

    fn encode_package(&self, package: &DamlPackage<'_>) -> Result<Vec<NamedSchema>> {
        self.encode_module(package.root_module().child_module_path_or_err(self.module_prefix)?)
    }

    fn encode_module(&self, module: &DamlModule<'_>) -> Result<Vec<NamedSchema>> {
        let mut result = Vec::new();
        for sub in module.child_modules() {
            result.extend(self.encode_module(sub)?);
        }
        for dt in module.data_types() {
            if self.filter_contains(dt)? && is_supported(dt) {
                result.push(self.encode_data(module, dt)?);
            }
        }
        Ok(result)
    }

    fn encode_data(&self, module: &DamlModule<'_>, data: &DamlData<'_>) -> Result<NamedSchema> {
        let name = format_oas_data(module, data);
        let schema = Schema::new(self.json_schema_encoder.encode_data(data)?);
        Ok(NamedSchema::new(name, schema))
    }

    /// Is the given data item referenced by the filter templates?
    ///
    /// If no filters are defined then all items are included.
    fn filter_contains(&self, needle: &DamlData<'_>) -> Result<bool> {
        self.filter.items.is_empty().not().then(|| self.check_filter(needle)).transpose().map(|o| o.unwrap_or(false))
    }

    fn check_filter(&self, needle: &DamlData<'_>) -> Result<bool> {
        let it = self.filter.items.iter().map(|(template_id, choice_filter)| {
            self.archive
                .data(self.archive.main_package_id(), &template_id.module, &template_id.entity)
                .ok_or_else(|| anyhow!("filter entity {} not found", template_id.to_string()))
                .map(|haystack| match haystack {
                    DamlData::Template(template) =>
                        DamlEntitySearcher::new(self.archive, needle).search_template(template, choice_filter),
                    _ => false,
                })
        });
        process_results(it, |mut ita| ita.any(identity))
    }
}

fn is_supported(dt: &DamlData<'_>) -> bool {
    dt.serializable() && dt.type_params().is_empty()
}
