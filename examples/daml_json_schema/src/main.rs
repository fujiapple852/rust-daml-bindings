#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::module_name_repetitions,
    clippy::use_self,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::cast_sign_loss
)]
#![forbid(unsafe_code)]

use anyhow::Result;
use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg};
use daml::json_api::schema_encoder::JsonSchemaEncoder;
use daml::lf::element::{DamlArchive, DamlElementVisitor, DamlModule, DamlRecord, DamlVisitableElement, DamlPackage, DamlData, DamlTemplate, DamlChoice};
use daml::lf::DarFile;
use itertools::Itertools;
use serde_json::{to_string_pretty, Value};
use daml::json_api::openapi_data::{PathItem, OpenAPI, Info, Paths, Components, Schema};
use maplit::btreemap;
use std::collections::BTreeMap;

fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("dar").help("Sets the input dar file to use").required(true).index(1))
        .get_matches();
    let dar_path = matches.value_of("dar").unwrap();
    execute(dar_path)
}

struct MyVisitor<'arc> {
    arc: &'arc DamlArchive<'arc>,
}

impl DamlElementVisitor for MyVisitor<'_> {
    fn pre_visit_module<'a>(&mut self, module: &'a DamlModule<'a>) {
        println!("=== {} === \n", module.path().join("."));
    }

    fn pre_visit_record<'a>(&mut self, record: &'a DamlRecord<'a>) {
        if !record.serializable() {
            println!("{} is not serializable\n", record.name());
            return;
        }
        let encoder = JsonSchemaEncoder::new(self.arc);
        match encoder.encode_record(record) {
            Ok(val) => println!("{}\n", to_string_pretty(&val).unwrap()),
            Err(err) => println!("failed for {}: {:?}", record.name(), err),
        }
    }
}

fn execute(dar_path: &str) -> Result<()> {
    let dar = DarFile::from_file(dar_path)?;
    Ok(dar.apply(|archive| {

        let renderer = OpenAPIRenderer { arc: archive, encoder: JsonSchemaEncoder::new(archive) };
        renderer.render_archive(archive)
        // render_archive(archive);


        // let mut vis = MyVisitor {
        //     arc: archive,
        // };
        // archive.main_package().unwrap().accept(&mut vis);
        // main_package(archive).unwrap().accept(&mut vis);
    })?)
}

struct OpenAPIRenderer<'arc> {
    arc: &'arc DamlArchive<'arc>,
    encoder: JsonSchemaEncoder<'arc>,
}

impl OpenAPIRenderer<'_> {

    fn render_archive(&self, archive: &DamlArchive<'_>) {
        self.render_package(archive.main_package().unwrap())
    }

    fn render_package(&self, package: &DamlPackage<'_>) {
        self.render_module_tree(package.root_module());
    }


    fn render_module_tree(&self, module: &DamlModule<'_>) {

        // module.data_types().for_each(|dt| match dt {
        //     DamlData::Template(template) => self.render_template(template),
        //     _ => {},
        // });

        module.child_modules().for_each(|m| self.render_module_tree(m));
        let module_path = module.path().join(".");
        println!("render module {}", module_path);
        let info = Info::new(format!("{}", module_path), self.arc.main_package().unwrap().version().unwrap());
        let paths = Paths::new(btreemap! {});

        // TODO need to filter data types we render t only those which are templates and choice params/results.

        let schemas: BTreeMap<String, Schema> = module
            .data_types()
            .filter_map(|dt| match dt {
                DamlData::Template(template) => {
                    println!("templ: {}, serial: {}", template.name(), template.serializable());
                    Some((template.name().to_owned(), Schema::new(self.encoder.encode_template(template).unwrap())))
                },
                _ => None
            })
            .collect();

        let components = Components::new(schemas);

        let openapi = OpenAPI::new(info, paths, components);
        println!("{}", serde_json::to_string_pretty(&openapi).unwrap());
    }

    // fn render_data(&self, data: &DamlData<'_>) -> Schema {
    //     match data {
    //         DamlData::Record(record) => Schema::new(self.encoder.encode_record(record).unwrap()),
    //         DamlData::Template(template)
    //     }
    // }

    // should generate a schema per module?

    // fn render_template(&self, template: &DamlTemplate<'_>) {
    //     println!("render template {}", template.name());
    //
    //     let info = Info::new(format!("{}", template.name()), self.arc.main_package().unwrap().version().unwrap());
    //
    //     let record_schema = self.encoder.encode_template(template).expect("encode_template failed");
    //
    //     // let choice_schemas: Vec<_> = template.choices().iter().map(|choice| {
    //     //
    //     //     choice.
    //     //
    //     // }).collect();
    //
    //     // TODO this is a OpenAPI PathItem for "/v1/create#DA.MyTemplate"
    //     // let create_path = self.render_create_path_item(template);
    //
    //     // TODO this is a list of OpenAPI PathItem for /v1/exercise#DA.MyTemplate (and by key etc)
    //     // let choices: Vec<_> = template.choices().iter().map(|c| self.render_choice_path_item(c)).collect();
    //
    //     let paths = Paths::new(btreemap! {});
    //
    //     // Schema
    //
    //     let components = Components::new(btreemap! {
    //         template.name().to_string() => Schema { value: record_schema }
    //     });
    //
    //     let openapi = OpenAPI::new(info, paths, components);
    //
    //     println!("{}", serde_json::to_string_pretty(&openapi).unwrap());
    //
    //
    // }

    // fn render_create_path_item(&self, template: &DamlTemplate<'_>) -> PathItem {
    //
    //
    //
    //     todo!()
    // }

    // fn render_choice_path_item(&self, choice: &DamlChoice<'_>) -> PathItem {
    //     todo!()
    // }
}

