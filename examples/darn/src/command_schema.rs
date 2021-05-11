use anyhow::Result;
use clap::{App, Arg, ArgMatches, SubCommand};
use itertools::Itertools;
use serde_json::to_string_pretty;

use daml::lf::element::{DamlArchive, DamlElementVisitor, DamlModule, DamlRecord, DamlVisitableElement};
use daml::lf::DarFile;

use crate::DarnCommand;
use daml::json_api::schema_encoder::JsonSchemaEncoder;

/// Darn command for displaying packages.
pub struct CommandSchema {}

impl DarnCommand for CommandSchema {
    fn name(&self) -> &str {
        "json-schema"
    }

    fn args<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name("json-schema")
            .about("Generate a JSON schema")
            .arg(Arg::with_name("dar").help("Sets the input dar file to use").required(true).index(1))
    }

    fn execute(&self, matches: &ArgMatches<'_>) -> Result<()> {
        let dar_path = matches.value_of("dar").unwrap();
        execute(dar_path)
    }
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
        archive.packages().find(|p| p.name() == "TestingTypes").unwrap().accept(&mut vis);

        // archive.accept(&mut vis);
    })?)
}
