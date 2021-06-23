use std::collections::BTreeMap;

use anyhow::Result;

use daml::json_api::schema_encoder::JsonSchemaEncoder;
use daml::lf::element::{DamlArchive, DamlData, DamlModule, DamlPackage};

use crate::common::NamedItem;
use crate::format::format_oas_data;
use crate::schema::Schema;
use crate::util::{ChildModulePathOrError, Required};

type NamedSchema = NamedItem<Schema>;

/// Encode Daml data types as JSON schema components.
///
/// This is designed to be used to generate OAS and A2S documents.
pub struct ComponentEncoder<'arc> {
    archive: &'arc DamlArchive<'arc>,
    module_prefix: &'arc [&'arc str],
    json_schema_encoder: &'arc JsonSchemaEncoder<'arc>,
}

impl<'arc> ComponentEncoder<'arc> {
    pub const fn new(
        archive: &'arc DamlArchive<'arc>,
        module_prefix: &'arc [&'arc str],
        json_schema_encoder: &'arc JsonSchemaEncoder<'arc>,
    ) -> Self {
        Self {
            archive,
            module_prefix,
            json_schema_encoder,
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

    fn encode_module(&self, module: &DamlModule<'_>) -> anyhow::Result<Vec<NamedSchema>> {
        let mut result = Vec::new();
        for sub in module.child_modules() {
            result.extend(self.encode_module(sub)?)
        }
        for dt in module.data_types() {
            if is_supported(dt) {
                result.push(self.encode_data(module, dt)?)
            }
        }
        Ok(result)
    }

    fn encode_data(&self, module: &DamlModule<'_>, data: &DamlData<'_>) -> Result<NamedSchema> {
        let name = format_oas_data(module, data);
        let schema = Schema::new(self.json_schema_encoder.encode_data(data)?);
        Ok(NamedSchema::new(name, schema))
    }
}

fn is_supported(dt: &DamlData<'_>) -> bool {
    dt.serializable() && dt.type_params().is_empty()
}
