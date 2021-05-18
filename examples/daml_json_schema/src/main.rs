#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![forbid(unsafe_code)]

use anyhow::Result;
use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg};
use itertools::Itertools;

use daml::json_api::schema_encoder::{JsonSchemaEncoder, RenderSchema, RenderTitle, SchemaEncoderConfig};
use daml::lf::element::{DamlArchive, DamlData, DamlModule, DamlPackage};
use daml::lf::DarFile;

fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("dar").help("Sets the input dar file to use").required(true).index(1))
        .arg(
            Arg::with_name("module-prefix")
                .short("m")
                .long("module")
                .takes_value(true)
                .required(false)
                .help("Module path prefix in the form Foo.Bar.Baz"),
        )
        .arg(
            Arg::with_name("show-schema")
                .short("s")
                .long("show-schema")
                .takes_value(true)
                .possible_values(&["none", "data", "all"])
                .default_value("none")
                .required(false)
                .help("include the `$schema` property"),
        )
        .arg(
            Arg::with_name("show-title")
                .short("t")
                .long("show-title")
                .takes_value(true)
                .possible_values(&["none", "data", "all"])
                .default_value("data")
                .required(false)
                .help("include the `title` property describing the types"),
        )
        .get_matches();
    let dar_path = matches.value_of("dar").unwrap();
    let module_prefix = matches.value_of("module-prefix").unwrap_or("");
    let render_schema = match matches.value_of("show-schema") {
        None => RenderSchema::None,
        Some(s) if s == "none" => RenderSchema::None,
        Some(s) if s == "data" => RenderSchema::Data,
        Some(s) if s == "all" => RenderSchema::All,
        Some(s) => panic!("unknown show-schema {}", s),
    };
    let render_title = match matches.value_of("show-title") {
        None => RenderTitle::None,
        Some(s) if s == "none" => RenderTitle::None,
        Some(s) if s == "data" => RenderTitle::Data,
        Some(s) if s == "all" => RenderTitle::All,
        Some(s) => panic!("unknown show-title {}", s),
    };
    let config = SchemaEncoderConfig::new(render_schema, render_title);
    let dar = DarFile::from_file(dar_path)?;
    dar.apply(|archive| {
        let renderer = SchemaRenderer {
            encoder: JsonSchemaEncoder::new_with_config(archive, config),
        };
        renderer.render_archive(archive, module_prefix)
    })?
}

struct SchemaRenderer<'arc> {
    encoder: JsonSchemaEncoder<'arc>,
}

impl SchemaRenderer<'_> {
    fn render_archive(&self, archive: &DamlArchive<'_>, module_prefix: &str) -> Result<()> {
        self.render_package(archive.main_package().expect("archive has no main package"), module_prefix)
    }

    fn render_package(&self, package: &DamlPackage<'_>, module_prefix: &str) -> Result<()> {
        println!("Package {} ({}):\n", package.name(), package.package_id());
        self.render_module_tree(package.root_module(), module_prefix)
    }

    fn render_module_tree(&self, module: &DamlModule<'_>, module_prefix: &str) -> Result<()> {
        for module in module.child_modules().sorted_by_key(|&m| m.path().collect::<Vec<_>>()) {
            self.render_module_tree(module, module_prefix)?;
        }
        if module.path().join(".").starts_with(module_prefix) {
            for dt in module.data_types().sorted_by_key(|dt| dt.name()) {
                if let DamlData::Template(template) = dt {
                    println!("Template {}.{}:\n", module.path().join("."), template.name());
                    println!("{}\n", serde_json::to_string_pretty(&self.encoder.encode_template(template)?)?);
                }
                if let DamlData::Record(record) = dt {
                    if !record.serializable() {
                        println!("Skipping Record {}.{} (not serializable)\n", module.path().join("."), record.name());
                        continue;
                    }
                    if !record.type_params().is_empty() {
                        println!(
                            "Skipping Record {}.{} (has generic type parameters)\n",
                            module.path().join("."),
                            record.name()
                        );
                        continue;
                    }
                    println!("Record {}.{}:\n", module.path().join("."), record.name());
                    println!("{}\n", serde_json::to_string_pretty(&self.encoder.encode_record(record)?)?);
                }
            }
        }
        Ok(())
    }
}
