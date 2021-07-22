use itertools::Itertools;

use daml::json_api::schema_encoder::JsonSchemaEncoder;
use daml::lf::element::{DamlArchive, DamlModule, DamlPackage};

use crate::common::ERROR_RESPONSE_SCHEMA_NAME;
use crate::companion::CompanionData;
use crate::component_encoder::ComponentEncoder;
use crate::config::PathStyle;
use crate::filter::TemplateFilter;
use crate::json_api_schema::DamlJsonApiSchema;
use crate::oas::openapi_data::{Components, Contact, Info, OpenAPI, Paths, Server, Tag};
use crate::oas::path_item_encoder::PathItemEncoder;
use crate::schema::Schema;
use crate::util::{ChildModulePathOrError, Required};

pub struct OpenAPIEncoder<'arc> {
    archive: &'arc DamlArchive<'arc>,
    module_path: &'arc [&'arc str],
    filter: &'arc TemplateFilter,
    reference_prefix: &'arc str,
    emit_package_id: bool,
    include_archive_choice: bool,
    path_style: PathStyle,
    companion_data: &'arc CompanionData,
    encoder: JsonSchemaEncoder<'arc>,
}

impl<'arc> OpenAPIEncoder<'arc> {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        archive: &'arc DamlArchive<'arc>,
        module_path: &'arc [&'arc str],
        filter: &'arc TemplateFilter,
        reference_prefix: &'arc str,
        emit_package_id: bool,
        include_archive_choice: bool,
        path_style: PathStyle,
        companion_data: &'arc CompanionData,
        encoder: JsonSchemaEncoder<'arc>,
    ) -> Self {
        Self {
            archive,
            module_path,
            filter,
            reference_prefix,
            emit_package_id,
            include_archive_choice,
            path_style,
            companion_data,
            encoder,
        }
    }

    /// Encode a `DamlArchive` as a JSON OAS document.
    pub fn encode_archive(&self) -> anyhow::Result<OpenAPI> {
        let info = self.encode_info()?;
        let servers = self.encode_servers();
        let paths = self.encode_paths()?;
        let components = self.encode_components()?;
        let tags = self.encode_tags()?;
        Ok(OpenAPI::new(info, servers, paths, components, tags))
    }

    fn encode_info(&self) -> anyhow::Result<Info> {
        let title =
            self.companion_data.title.as_ref().map_or_else(|| self.archive.name().to_string(), ToString::to_string);
        let description = self
            .companion_data
            .description
            .as_ref()
            .map(ToString::to_string)
            .or_else(|| Some(format!("OpenAPI specification for Daml archive {}", self.archive.name())));
        let contact_name = self
            .companion_data
            .contact
            .as_ref()
            .and_then(|c| c.name.as_ref())
            .map(ToString::to_string)
            .or_else(|| Some(format!("")));
        let url = self
            .companion_data
            .contact
            .as_ref()
            .and_then(|c| c.url.as_ref())
            .map(ToString::to_string)
            .or_else(|| Some(format!("")));
        let email = self
            .companion_data
            .contact
            .as_ref()
            .and_then(|c| c.email.as_ref())
            .map(ToString::to_string)
            .or_else(|| Some(format!("")));
        let contact = Contact::new(contact_name, url, email);
        let version = self
            .companion_data
            .version
            .as_deref()
            .or_else(|| self.archive.main_package().and_then(DamlPackage::version))
            .req()?;
        Ok(Info::new(title, self.companion_data.summary.clone(), Some(contact), description, version))
    }

    fn encode_servers(&self) -> Vec<Server> {
        self.companion_data
            .servers
            .as_ref()
            .unwrap_or(&Vec::default())
            .iter()
            .map(|s| Server::new(s.to_string(), None))
            .collect()
    }

    fn encode_paths(&self) -> anyhow::Result<Paths> {
        Ok(Paths::new(
            PathItemEncoder::new(
                self.archive,
                self.module_path,
                self.filter,
                self.reference_prefix,
                self.emit_package_id,
                self.include_archive_choice,
                self.path_style,
                self.companion_data,
                &self.encoder,
            )
            .encode_path_items()?,
        ))
    }

    fn encode_components(&self) -> anyhow::Result<Components> {
        let encoder = ComponentEncoder::new(self.archive, self.module_path, &self.encoder, self.filter);
        let mut schemas = encoder.encode_schema_components()?;
        schemas.insert(ERROR_RESPONSE_SCHEMA_NAME.to_string(), Schema::new(DamlJsonApiSchema::make_error_response()));
        Ok(Components::new(schemas))
    }

    fn encode_tags(&self) -> anyhow::Result<Vec<Tag>> {
        let root = self.archive.main_package().req()?.root_module().child_module_path_or_err(self.module_path)?;
        Ok(self.module_path(root))
    }

    fn module_path(&self, module: &DamlModule<'_>) -> Vec<Tag> {
        Self::has_data(module)
            .then(|| Tag::new(module.path().join("."), None))
            .into_iter()
            .chain(module.child_modules().flat_map(|cm| self.module_path(cm)))
            .collect()
    }

    fn has_data(module: &DamlModule<'_>) -> bool {
        module.data_types().next().is_some()
    }
}
