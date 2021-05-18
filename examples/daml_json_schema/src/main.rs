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
use daml::lf::element::{DamlArchive, DamlElementVisitor, DamlModule, DamlRecord, DamlVisitableElement};
use daml::lf::DarFile;
use itertools::Itertools;
use serde_json::to_string_pretty;

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
        let mut vis = MyVisitor {
            arc: archive,
        };
        archive.main_package().unwrap().accept(&mut vis);
        // main_package(archive).unwrap().accept(&mut vis);
    })?)
}

