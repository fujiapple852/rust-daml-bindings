use std::collections::BTreeMap;

use maplit::btreemap;
use serde_json::Value;

use daml::json_api::schema_encoder::JsonSchemaEncoder;
use daml::lf::element::{
    DamlArchive, DamlChoice, DamlData, DamlDefKey, DamlModule, DamlPackage, DamlTemplate, DamlTyCon,
};

use crate::choice_event_extractor::ChoiceEventExtractor;
use crate::common::{DataId, NamedItem, ARCHIVE_CHOICE_NAME, ERROR_RESPONSE_SCHEMA_NAME};
use crate::format;
use crate::format::{format_daml_template, format_path};
use crate::json_api_schema::DamlJsonApiSchema;
use crate::openapi_data::{MediaType, Operation, PathItem, RequestBody, Response, ResponseType, Responses, Schema};
use crate::operation::OperationIdFactory;
use crate::operation::PathStyle;
use crate::util::{ChildModulePathOrError, Required};
use itertools::{chain, process_results};

type NamedPathItem = NamedItem<PathItem>;

/// DOCME
pub struct PathItemEncoder<'arc> {
    archive: &'arc DamlArchive<'arc>,
    module_path: &'arc [&'arc str],
    include_archive_choice: bool,
    operation_id_factory: OperationIdFactory,
    json_type_schema_encoder: &'arc JsonSchemaEncoder<'arc>,
    json_api_schema: DamlJsonApiSchema,
}

impl<'arc> PathItemEncoder<'arc> {
    pub fn new(
        archive: &'arc DamlArchive<'arc>,
        module_path: &'arc [&'arc str],
        reference_prefix: &'arc str,
        emit_package_id: bool,
        include_archive_choice: bool,
        path_style: PathStyle,
        json_type_schema_encoder: &'arc JsonSchemaEncoder<'arc>,
    ) -> Self {
        Self {
            archive,
            module_path,
            include_archive_choice,
            operation_id_factory: OperationIdFactory::new(path_style),
            json_type_schema_encoder,
            json_api_schema: DamlJsonApiSchema::new(reference_prefix, emit_package_id),
        }
    }

    ///
    pub fn encode_path_items(self) -> anyhow::Result<BTreeMap<String, PathItem>> {
        Ok(self
            .encode_package(self.archive.main_package().req()?)?
            .into_iter()
            .map(|pi| (pi.name, pi.item))
            .collect::<BTreeMap<String, PathItem>>())
    }

    fn encode_package(&self, package: &DamlPackage<'_>) -> anyhow::Result<Vec<NamedPathItem>> {
        self.encode_module(package, package.root_module().child_module_path_or_err(self.module_path)?)
    }

    fn encode_module(&self, package: &DamlPackage<'_>, module: &DamlModule<'_>) -> anyhow::Result<Vec<NamedPathItem>> {
        let dt_iter = module.data_types().map(|dt| self.encode_data(package, module, dt));
        let child_mod_iter = module.child_modules().map(|m| self.encode_module(package, m));
        process_results(chain(dt_iter, child_mod_iter), |ita| ita.flatten().collect::<Vec<_>>())
    }

    fn encode_data(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        data: &DamlData<'_>,
    ) -> anyhow::Result<Vec<NamedPathItem>> {
        Ok(if let DamlData::Template(template) = data {
            self.encode_template(package, module, template)?
        } else {
            vec![]
        })
    }

    fn encode_template(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
    ) -> anyhow::Result<Vec<NamedPathItem>> {
        let create = self.encode_template_create(package, module, template);
        let create_and_exercise = self.encode_template_create_and_exercise(package, module, template)?;
        let exercise_by_id = self.encode_template_exercise_by_id(package, module, template)?;
        let exercise_by_key = self.encode_template_exercise_by_key(package, module, template)?;
        let fetch_by_id = self.encode_template_fetch_by_id(package, module, template);
        let fetch_by_key =
            template.key().map(|key| self.encode_template_fetch_by_key(package, module, template, key)).transpose()?;
        Ok(std::iter::once(create)
            .into_iter()
            .chain(create_and_exercise)
            .chain(exercise_by_id)
            .chain(exercise_by_key)
            .chain(std::iter::once(fetch_by_id))
            .chain(fetch_by_key)
            .collect())
    }

    fn encode_template_create(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
    ) -> NamedPathItem {
        let template_id = make_template_id(package, module, template);
        let operation_id = self.operation_id_factory.create_by_id(&template_id);
        let tags = make_tags(&template_id);
        let summary = format!("Create a contract of the {} template", format_daml_template(&template_id));
        let request = self.json_api_schema.make_create_request(&template_id);
        let response = self.json_api_schema.make_create_response(&template_id);
        let path_item = self.make_path_item(summary, operation_id.clone(), tags, request, response);
        NamedPathItem::new(operation_id, path_item)
    }

    fn encode_template_exercise_by_id(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
    ) -> anyhow::Result<Vec<NamedPathItem>> {
        template
            .choices()
            .iter()
            .filter_map(|choice| {
                self.should_include_choice(choice)
                    .then(|| self.encode_template_exercise_by_id_choice(package, module, template, choice))
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }

    fn encode_template_exercise_by_key(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
    ) -> anyhow::Result<Vec<NamedPathItem>> {
        template
            .choices()
            .iter()
            .filter_map(|choice| match template.key() {
                Some(key) if self.should_include_choice(choice) =>
                    Some(self.encode_template_exercise_by_key_choice(package, module, template, choice, key)),
                _ => None,
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }

    fn encode_template_create_and_exercise(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
    ) -> anyhow::Result<Vec<NamedPathItem>> {
        template
            .choices()
            .iter()
            .filter_map(|choice| {
                self.should_include_choice(choice)
                    .then(|| self.encode_template_create_and_exercise_choice(package, module, template, choice))
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }

    fn encode_template_fetch_by_id(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
    ) -> NamedPathItem {
        let template_id = make_template_id(package, module, template);
        let summary = format!("Fetch a contract of the {} template by contract id", format_daml_template(&template_id));
        let operation_id = self.operation_id_factory.fetch_by_id(&template_id);
        let tags = make_tags(&template_id);
        let request = DamlJsonApiSchema::make_fetch_by_id_request();
        let response = self.json_api_schema.make_fetch_response(&template_id);
        let path_item = self.make_path_item(summary, operation_id.clone(), tags, request, response);
        NamedPathItem::new(operation_id, path_item)
    }

    fn encode_template_fetch_by_key(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
        key: &DamlDefKey<'_>,
    ) -> anyhow::Result<NamedPathItem> {
        let template_id = make_template_id(package, module, template);
        let key_encoded = self.json_type_schema_encoder.encode_type(key.ty())?;
        let summary = format!("Fetch a contract of the {} template by key", format_daml_template(&template_id));
        let operation_id = self.operation_id_factory.fetch_by_key(&template_id);
        let tags = make_tags(&template_id);
        let request = self.json_api_schema.make_fetch_by_key_request(&template_id, &key_encoded);
        let response = self.json_api_schema.make_fetch_response(&template_id);
        let path_item = self.make_path_item(summary, operation_id.clone(), tags, request, response);
        Ok(NamedPathItem::new(operation_id, path_item))
    }

    fn encode_template_exercise_by_id_choice(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
        choice: &DamlChoice<'_>,
    ) -> anyhow::Result<NamedPathItem> {
        let template_id = make_template_id(package, module, template);
        let return_type_ref = self.json_type_schema_encoder.encode_type(choice.return_type())?;
        let (created, archived) = self.extract_exercise_events(package, module, template, choice);
        let summary = format!(
            "Exercise the {} choice on a contract of the {} template by contract id",
            choice.name(),
            format_daml_template(&template_id)
        );
        let operation_id = self.operation_id_factory.exercise_by_id(&template_id, choice.name());
        let tags = make_tags(&template_id);
        let args = self.make_args(&template_id, choice.name());
        let request = self.json_api_schema.make_exercise_by_id_request(&template_id, choice.name(), &args);
        let response = self.json_api_schema.make_exercise_response(&return_type_ref, &created, &archived);
        let path_item = self.make_path_item(summary, operation_id.clone(), tags, request, response);
        Ok(NamedPathItem::new(operation_id, path_item))
    }

    fn encode_template_exercise_by_key_choice(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
        choice: &DamlChoice<'_>,
        key: &DamlDefKey<'_>,
    ) -> anyhow::Result<NamedPathItem> {
        let template_id = make_template_id(package, module, template);
        let return_type_ref = self.json_type_schema_encoder.encode_type(choice.return_type())?;
        let (created, archived) = self.extract_exercise_events(package, module, template, choice);
        let key_encoded = self.json_type_schema_encoder.encode_type(key.ty())?;
        let summary = format!(
            "Exercise the {} choice on a contract of the {} template by contract key",
            choice.name(),
            format_daml_template(&template_id),
        );
        let operation_id = self.operation_id_factory.exercise_by_key(&template_id, choice.name());
        let tags = make_tags(&template_id);
        let args = self.make_args(&template_id, choice.name());
        let request =
            self.json_api_schema.make_exercise_by_key_request(&template_id, choice.name(), &args, &key_encoded);
        let response = self.json_api_schema.make_exercise_response(&return_type_ref, &created, &archived);
        let path_item = self.make_path_item(summary, operation_id.clone(), tags, request, response);
        Ok(NamedPathItem::new(operation_id, path_item))
    }

    fn encode_template_create_and_exercise_choice(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
        choice: &DamlChoice<'_>,
    ) -> anyhow::Result<NamedPathItem> {
        let template_id = make_template_id(package, module, template);
        let return_type_ref = self.json_type_schema_encoder.encode_type(choice.return_type())?;
        let (created, archived) = self.extract_create_and_exercise_events(package, module, template, choice);
        let summary = format!(
            "Create a contract of the {} template and immediately exercise the {} choice on it",
            format_daml_template(&template_id),
            choice.name(),
        );
        let operation_id = self.operation_id_factory.create_and_exercise(&template_id, choice.name());
        let tags = make_tags(&template_id);
        let args = self.make_args(&template_id, choice.name());
        let request = self.json_api_schema.make_create_and_exercise_request(&template_id, choice.name(), &args);
        let response = self.json_api_schema.make_exercise_response(&return_type_ref, &created, &archived);
        let path_item = self.make_path_item(summary, operation_id.clone(), tags, request, response);
        Ok(NamedPathItem::new(operation_id, path_item))
    }

    fn extract_exercise_events(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
        choice: &DamlChoice<'_>,
    ) -> (Vec<DataId>, Vec<DataId>) {
        self.extract_events(package, module, template, choice, false)
    }

    fn extract_create_and_exercise_events(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
        choice: &DamlChoice<'_>,
    ) -> (Vec<DataId>, Vec<DataId>) {
        self.extract_events(package, module, template, choice, true)
    }

    fn extract_events(
        &self,
        package: &DamlPackage<'_>,
        module: &DamlModule<'_>,
        template: &DamlTemplate<'_>,
        choice: &DamlChoice<'_>,
        include_creating_template: bool,
    ) -> (Vec<DataId>, Vec<DataId>) {
        let events = self.archive.extract_choice_events(package, module, template, choice);
        let created = include_creating_template
            .then(|| make_template_id(package, module, template))
            .into_iter()
            .chain(events.created().map(make_template_id_from_tycon))
            .collect::<Vec<_>>();
        let archived = events.archived().map(make_template_id_from_tycon).collect::<Vec<_>>();
        (created, archived)
    }

    fn make_path_item(
        &self,
        summary: String,
        operation_id: String,
        tags: Vec<String>,
        request: Value,
        response: Value,
    ) -> PathItem {
        let body = Self::make_json_request_body(request);
        let responses = self.make_json_responses_with_error(response);
        let post_operation = Operation::new(Some(summary), Some(operation_id), tags, body, responses);
        PathItem::new(Some(post_operation))
    }

    fn make_args(&self, template_id: &DataId, choice: &str) -> Value {
        if choice == ARCHIVE_CHOICE_NAME {
            DamlJsonApiSchema::make_archive_type()
        } else {
            self.json_api_schema.make_schema_ref(&format::format_oas_template_choice(template_id, choice))
        }
    }

    fn make_json_responses_with_error(&self, success_response: Value) -> Responses {
        Responses {
            default: Some(self.make_json_error_response()),
            responses: btreemap! { "200".to_string() => Self::make_json_success_response(success_response) },
        }
    }

    fn make_json_error_response(&self) -> ResponseType {
        Self::make_json_content_response(
            "error".to_string(),
            self.json_api_schema.make_schema_ref(ERROR_RESPONSE_SCHEMA_NAME),
        )
    }

    fn make_json_request_body(request: Value) -> RequestBody {
        RequestBody {
            description: None,
            content: Self::make_json_content_map(request),
            required: None,
        }
    }

    fn make_json_success_response(schema: Value) -> ResponseType {
        Self::make_json_content_response("success".to_string(), schema)
    }

    fn make_json_content_response(description: String, schema: Value) -> ResponseType {
        ResponseType::Response(Response {
            description,
            content: Self::make_json_content_map(schema),
        })
    }

    fn make_json_content_map(schema: Value) -> BTreeMap<String, MediaType> {
        btreemap! {
            "application/json".to_string() => MediaType::new(Schema::new(schema))
        }
    }

    fn should_include_choice(&self, choice: &DamlChoice<'_>) -> bool {
        return self.include_archive_choice || (choice.name() != ARCHIVE_CHOICE_NAME);
    }
}

fn make_tags(template_id: &DataId) -> Vec<String> {
    vec![format_path(&template_id.module_path)]
}

fn make_template_id(package: &DamlPackage<'_>, module: &DamlModule<'_>, template: &DamlTemplate<'_>) -> DataId {
    DataId::new(
        package.package_id().to_string(),
        module.path().map(ToString::to_string).collect(),
        template.name().to_string(),
    )
}

fn make_template_id_from_tycon(tycon: &DamlTyCon<'_>) -> DataId {
    DataId::new(
        tycon.tycon().package_id().to_string(),
        tycon.tycon().module_path().map(ToString::to_string).collect(),
        tycon.tycon().data_name().to_string(),
    )
}
