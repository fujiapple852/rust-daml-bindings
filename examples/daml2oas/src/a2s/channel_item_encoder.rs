use std::collections::BTreeMap;

use maplit::btreemap;
use serde_json::Value;

use bounded_static::ToBoundedStatic;
use daml::json_api::error::DamlJsonSchemaCodecResult;
use daml::json_api::schema_encoder::JsonSchemaEncoder;
use daml::lf::element::{DamlArchive, DamlElementVisitor, DamlTemplate, DamlType, DamlVisitableElement};

use crate::a2s::asyncapi_data::{
    ChannelBindings, ChannelItem, ExternalDocumentation, Message, OneOfMessages, Operation, Tag,
    WebSocketsChannelBinding,
};
use crate::common::DataId;
use crate::filter::TemplateFilter;
use crate::json_api_schema::DamlJsonApiSchema;
use crate::schema::Schema;
use crate::util::{ChildModulePathOrError, Required};

/// DOCME
pub struct ChannelItemEncoder<'arc> {
    archive: &'arc DamlArchive<'arc>,
    module_path: &'arc [&'arc str],
    filter: &'arc TemplateFilter,
    json_type_schema_encoder: &'arc JsonSchemaEncoder<'arc>,
    json_api_schema: DamlJsonApiSchema,
}

impl<'arc> ChannelItemEncoder<'arc> {
    pub fn new(
        archive: &'arc DamlArchive<'arc>,
        module_path: &'arc [&'arc str],
        filter: &'arc TemplateFilter,
        reference_prefix: &'arc str,
        emit_package_id: bool,
        json_type_schema_encoder: &'arc JsonSchemaEncoder<'arc>,
    ) -> Self {
        Self {
            archive,
            module_path,
            filter,
            json_type_schema_encoder,
            json_api_schema: DamlJsonApiSchema::new(reference_prefix, emit_package_id),
        }
    }

    ///
    pub fn encode_channel_items(self) -> anyhow::Result<BTreeMap<String, ChannelItem>> {
        let template_ids = self.extract_template_info()?;
        Ok(btreemap! {
            String::from("/v1/stream/query") => self.encode_query_channel_item(&template_ids),
            String::from("/v1/stream/fetch") => self.encode_fetch_channel_item(&template_ids)?,
        })
    }

    fn encode_query_channel_item(&self, templates: &[TemplateData]) -> ChannelItem {
        let template_ids = templates
            .iter()
            .cloned()
            .filter_map(|t| self.filter_contains(&t.template_id).then(|| t.template_id))
            .collect::<Vec<_>>();
        let single_request = self.encode_query_single_request(&template_ids);
        let multi_request = self.encode_query_multi_request(&template_ids);
        let offset_request = Self::encode_query_offset_request();
        let events = self.encode_query_ledger_events(&template_ids);
        let warnings = Self::encode_ledger_warnings();
        let errors = Self::encode_ledger_errors();
        let publish_messages = OneOfMessages::new(vec![single_request, multi_request, offset_request]);
        let subscribe_messages = OneOfMessages::new(vec![events, warnings, errors]);
        ChannelItem::new(
            String::from("List currently active contracts that match a given query, with continuous updates."),
            Operation::new(String::from("queryPublish"), publish_messages),
            Operation::new(String::from("querySubscribe"), subscribe_messages),
            ChannelBindings::new(WebSocketsChannelBinding::default()),
        )
    }

    fn encode_fetch_channel_item(&self, templates: &[TemplateData]) -> anyhow::Result<ChannelItem> {
        let template_ids = templates
            .iter()
            .cloned()
            .filter_map(|t| self.filter_contains(&t.template_id).then(|| t))
            .filter(|t| t.key.is_some())
            .map(|t| t.template_id)
            .collect::<Vec<_>>();
        let template_keys = templates
            .iter()
            .filter(|t| t.key.is_some())
            .filter_map(|t| t.key.as_ref().map(|k| self.json_type_schema_encoder.encode_type(k)))
            .collect::<DamlJsonSchemaCodecResult<Vec<_>>>()?;
        let request = self.encode_fetch_request(&template_ids, &template_keys);
        let events = self.encode_fetch_ledger_events(&template_ids);
        let warnings = Self::encode_ledger_warnings();
        let errors = Self::encode_ledger_errors();
        let publish_messages = OneOfMessages::new(vec![request]);
        let subscribe_messages = OneOfMessages::new(vec![events, warnings, errors]);
        Ok(ChannelItem::new(
            String::from(
                "List currently active contracts that match one of the given {templateId, key} pairs, with continuous \
                 updates.",
            ),
            Operation::new(String::from("fetchPublish"), publish_messages),
            Operation::new(String::from("fetchSubscribe"), subscribe_messages),
            ChannelBindings::new(WebSocketsChannelBinding::default()),
        ))
    }

    fn encode_query_single_request(&self, template_ids: &[DataId]) -> Message {
        let payload = Schema::new(self.json_api_schema.make_stream_query_single_request(template_ids));
        Message::new(
            String::from("queryRequest"),
            String::from("Query Request"),
            String::from("Subscribe to ledger events for a given query"),
            String::from("The body must be sent first, formatted according to the [Query language](https://docs.daml.com/json-api/search-query-language.html)"),
            String::from("application/json"),
            payload,
            vec![make_query_tag()],
        )
    }

    fn encode_query_multi_request(&self, template_ids: &[DataId]) -> Message {
        let payload = Schema::new(self.json_api_schema.make_stream_query_multi_request(template_ids));
        Message::new(
            String::from("queryMultiRequest"),
            String::from("Multi Query Request"),
            String::from("Subscribe to ledger events for the given queries"),
            String::from(
                "Multiple queries may be specified in an array, for overlapping or different sets of template IDs",
            ),
            String::from("application/json"),
            payload,
            vec![make_query_tag()],
        )
    }

    fn encode_query_offset_request() -> Message {
        let payload = Schema::new(DamlJsonApiSchema::make_stream_query_offset_request());
        Message::new(
            String::from("queryOffsetRequest"),
            String::from("Query Offset Request"),
            String::from("The offset from which to begin streaming"),
            String::from(
                "An optional offset returned by a prior query may be specified before the above, as a separate body. \
                 It must be a string, and if specified, the stream will begin immediately after the response body \
                 that included that offset.",
            ),
            String::from("application/json"),
            payload,
            vec![make_query_tag()],
        )
    }

    fn encode_ledger_warnings() -> Message {
        Message::new(
            String::from("queryLedgerWarnings"),
            String::from("Ledger Warnings"),
            String::from("The ledger warnings"),
            String::from("The Ledger warnings generated by the streaming request"),
            String::from("application/json"),
            Schema::new(DamlJsonApiSchema::make_stream_warnings()),
            vec![],
        )
    }

    fn encode_ledger_errors() -> Message {
        Message::new(
            String::from("queryLedgerErrors"),
            String::from("Ledger Errors"),
            String::from("The ledger errors"),
            String::from("The Ledger errors generated by the streaming request"),
            String::from("application/json"),
            Schema::new(DamlJsonApiSchema::make_stream_errors()),
            vec![],
        )
    }

    fn encode_query_ledger_events(&self, template_ids: &[DataId]) -> Message {
        let payload = self.json_api_schema.make_stream_events_response(template_ids, template_ids);
        Message::new(
            String::from("queryLedgerEvents"),
            String::from("Ledger Events for Query"),
            String::from("The ledger events for a given ledger offset"),
            String::from("The output is a series of JSON documents, each payload formatted according to [Daml-LF JSON Encoding](https://docs.daml.com/json-api/lf-value-specification.html)"),
            String::from("application/json"),
            Schema::new(payload),
            vec![make_query_tag()],
        )
    }

    fn encode_fetch_request(&self, template_ids: &[DataId], keys: &[Value]) -> Message {
        let payload = Schema::new(self.json_api_schema.make_stream_fetch_request(template_ids, keys));
        Message::new(
            String::from("fetchRequest"),
            String::from("Fetch Request"),
            String::from("Subscribe to ledger events for the given templates and keys"),
            String::from("The body must be sent first"),
            String::from("application/json"),
            payload,
            vec![make_fetch_tag()],
        )
    }

    fn encode_fetch_ledger_events(&self, template_ids: &[DataId]) -> Message {
        let payload = self.json_api_schema.make_stream_events_response(template_ids, template_ids);
        Message::new(
            String::from("fetchLedgerEvents"),
            String::from("Ledger Events for Fetch"),
            String::from("The ledger events for a given ledger offset"),
            String::from("The output is a series of JSON documents, each payload formatted according to [Daml-LF JSON Encoding](https://docs.daml.com/json-api/lf-value-specification.html)"),
            String::from("application/json"),
            Schema::new(payload),
            vec![make_fetch_tag()],
        )
    }

    fn extract_template_info(&self) -> anyhow::Result<Vec<TemplateData>> {
        struct TemplateVisitor {
            pub templates: Vec<TemplateData>,
        }
        impl DamlElementVisitor for TemplateVisitor {
            fn pre_visit_template<'a>(&mut self, template: &'a DamlTemplate<'a>) {
                let template_id = make_template_id(template.package_id(), template.module_path(), template.name());
                let key = template.key().map(|k| k.ty().to_static());
                self.templates.push(TemplateData::new(template_id, key));
            }
        }
        let mut visitor = TemplateVisitor {
            templates: Vec::default(),
        };
        self.archive
            .main_package()
            .req()?
            .root_module()
            .child_module_path_or_err(self.module_path)?
            .accept(&mut visitor);
        Ok(visitor.templates)
    }

    /// Return true if the filter contains this template, false otherwise.
    ///
    /// If the filter is empty then true is returned.
    fn filter_contains(&self, template: &DataId) -> bool {
        fn package(item: &DataId, template: &DataId) -> bool {
            match (item.package_id.as_ref(), template.package_id.as_ref()) {
                (Some(i), Some(p)) => i == p,
                _ => true,
            }
        }
        fn name(item: &DataId, template: &DataId) -> bool {
            item.entity == template.entity
        }
        fn module(item: &DataId, template: &DataId) -> bool {
            item.module.iter().zip(template.module.iter()).all(|(x, y)| x == y)
        }
        self.filter.items.is_empty()
            || self
                .filter
                .items
                .keys()
                .any(|item| package(item, template) && module(item, template) && name(item, template))
    }
}

fn make_template_id<'a>(package_id: &str, module_path: impl Iterator<Item = &'a str>, name: &str) -> DataId {
    DataId::new(Some(package_id.to_string()), module_path.map(ToString::to_string).collect(), name.to_string())
}

fn make_query_tag() -> Tag {
    Tag::new(
        String::from("query"),
        Some(String::from("Contract Query Stream")),
        Some(ExternalDocumentation::new(
            None,
            String::from("https://docs.daml.com/json-api/index.html#contracts-query-stream"),
        )),
    )
}

fn make_fetch_tag() -> Tag {
    Tag::new(
        String::from("fetch"),
        Some(String::from("Fetch By Key Contract Stream")),
        Some(ExternalDocumentation::new(
            None,
            String::from("https://docs.daml.com/json-api/index.html#fetch-by-key-contracts-stream"),
        )),
    )
}

#[derive(Debug, Clone)]
struct TemplateData {
    template_id: DataId,
    key: Option<DamlType<'static>>,
}

impl TemplateData {
    pub const fn new(template_id: DataId, key: Option<DamlType<'static>>) -> Self {
        Self {
            template_id,
            key,
        }
    }
}
