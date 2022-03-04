use daml::json_api::schema_encoder::JsonSchemaEncoder;
use daml::lf::element::{DamlArchive, DamlPackage};

use crate::a2s::asyncapi_data::{AsyncAPI, Channels, Components, Info, Server, Servers};
use crate::a2s::channel_item_encoder::ChannelItemEncoder;
use crate::companion::CompanionData;
use crate::component_encoder::ComponentEncoder;
use crate::filter::TemplateFilter;
use crate::util::Required;
use std::collections::BTreeMap;

pub struct AsyncAPIEncoder<'arc> {
    archive: &'arc DamlArchive<'arc>,
    module_path: &'arc [&'arc str],
    filter: &'arc TemplateFilter,
    reference_prefix: &'arc str,
    emit_package_id: bool,
    companion_data: &'arc CompanionData,
    encoder: JsonSchemaEncoder<'arc>,
}

impl<'arc> AsyncAPIEncoder<'arc> {
    pub const fn new(
        archive: &'arc DamlArchive<'arc>,
        module_path: &'arc [&'arc str],
        filter: &'arc TemplateFilter,
        reference_prefix: &'arc str,
        emit_package_id: bool,
        companion_data: &'arc CompanionData,
        encoder: JsonSchemaEncoder<'arc>,
    ) -> Self {
        Self {
            archive,
            module_path,
            filter,
            reference_prefix,
            emit_package_id,
            companion_data,
            encoder,
        }
    }

    /// Encode a `DamlArchive` as a JSON A2S document.
    pub fn encode_archive(&self) -> anyhow::Result<AsyncAPI> {
        let info = self.encode_info()?;
        let servers = self.encode_servers();
        let channels = self.encode_channels()?;
        let components = self.encode_components()?;
        Ok(AsyncAPI::new(info, servers, channels, components))
    }

    fn encode_info(&self) -> anyhow::Result<Info> {
        let title =
            self.companion_data.title.as_ref().map_or_else(|| self.archive.name().to_string(), ToString::to_string);
        let version = self
            .companion_data
            .version
            .as_deref()
            .or_else(|| self.archive.main_package().and_then(DamlPackage::version))
            .req()?;
        let description = self
            .companion_data
            .description
            .as_ref()
            .map(ToString::to_string)
            .or_else(|| Some(format!("AsyncAPI specification for Daml archive {}", self.archive.name())));
        Ok(Info::new(title, version.to_string(), description))
    }

    fn encode_servers(&self) -> Servers {
        let servers = self
            .companion_data
            .servers
            .as_ref()
            .unwrap_or(&Vec::default())
            .iter()
            .map(|s| (String::from("default"), Server::new(s.to_string(), String::from("ws"))))
            .collect::<BTreeMap<_, _>>();
        Servers::new(servers)
    }

    fn encode_channels(&self) -> anyhow::Result<Channels> {
        Ok(Channels::new(
            ChannelItemEncoder::new(
                self.archive,
                self.module_path,
                self.filter,
                self.reference_prefix,
                self.emit_package_id,
                &self.encoder,
            )
            .encode_channel_items()?,
        ))
    }

    fn encode_components(&self) -> anyhow::Result<Components> {
        let encoder = ComponentEncoder::new(self.archive, self.module_path, &self.encoder, self.filter);
        let schemas = encoder.encode_schema_components()?;
        Ok(Components::new(schemas))
    }
}
