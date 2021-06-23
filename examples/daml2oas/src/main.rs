#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate, clippy::missing_errors_doc)]
#![forbid(unsafe_code)]

use crate::a2s::AsyncAPI;
use crate::a2s::AsyncAPIEncoder;
use anyhow::{Context, Result};
use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand};
use companion::CompanionData;
use config::PathStyle;
use config::{Config, OutputFormat};
use daml::json_api::schema_encoder::{
    JsonSchemaEncoder, ReferenceMode, RenderSchema, RenderTitle, SchemaEncoderConfig,
};
use daml::lf::DarFile;
use oas::OpenAPI;
use oas::OpenAPIEncoder;
use serde::Serialize;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

mod a2s;
mod common;
mod companion;
mod component_encoder;
mod config;
mod format;
mod json_api_schema;
mod oas;
mod schema;
mod util;

fn main() -> Result<()> {
    let oas = SubCommand::with_name("oas")
        .about("Generate an OpenAPI document from the given Dar file")
        .arg(make_dar_arg())
        .arg(make_format_arg())
        .arg(make_output_arg())
        .arg(make_companion_file_arg())
        .arg(make_module_path_arg())
        .arg(make_describe_type_arg())
        .arg(make_reference_prefix_arg())
        .arg(make_reference_mode_arg())
        .arg(make_include_package_id_arg())
        .arg(make_include_archive_choice_arg())
        .arg(make_path_style_arg());
    let a2s = SubCommand::with_name("a2s")
        .about("Generate an AsyncAPI document from the given Dar file")
        .arg(make_dar_arg())
        .arg(make_format_arg())
        .arg(make_output_arg())
        .arg(make_companion_file_arg())
        .arg(make_module_path_arg())
        .arg(make_describe_type_arg())
        .arg(make_reference_prefix_arg())
        .arg(make_reference_mode_arg())
        .arg(make_include_package_id_arg());
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .set_term_width(0)
        .subcommand(oas)
        .subcommand(a2s)
        .get_matches();
    match matches.subcommand() {
        ("oas", Some(sub)) => execute_oas(&parse_config(sub))?,
        ("a2s", Some(sub)) => execute_a2s(&parse_config(sub))?,
        _ => {},
    };
    Ok(())
}

fn make_dar_arg() -> Arg<'static, 'static> {
    Arg::with_name("dar").help("Sets the input dar file to use").required(true).index(1)
}

fn make_format_arg() -> Arg<'static, 'static> {
    Arg::with_name("format")
        .short("f")
        .long("format")
        .takes_value(true)
        .possible_values(&["json", "yaml"])
        .default_value("json")
        .required(false)
        .help("the output format")
}

fn make_output_arg() -> Arg<'static, 'static> {
    Arg::with_name("output").short("o").long("output").takes_value(true).required(false).help("the output file path")
}

fn make_companion_file_arg() -> Arg<'static, 'static> {
    Arg::with_name("companion-file")
        .short("c")
        .long("companion-file")
        .takes_value(true)
        .default_value(".daml2oas.yaml")
        .required(false)
        .help("the companion yaml file with auxiliary data to inject into the generated OAS document")
}

fn make_module_path_arg() -> Arg<'static, 'static> {
    Arg::with_name("module-path")
        .short("m")
        .long("module")
        .takes_value(true)
        .required(false)
        .help("module path prefix in the form Foo.Bar.Baz")
}

fn make_describe_type_arg() -> Arg<'static, 'static> {
    Arg::with_name("describe-types")
        .short("t")
        .long("describe-types")
        .takes_value(true)
        .possible_values(&["none", "data", "all"])
        .default_value("all")
        .required(false)
        .help("include the `title` property describing the types")
}

fn make_reference_prefix_arg() -> Arg<'static, 'static> {
    Arg::with_name("reference-prefix")
        .short("p")
        .long("reference-prefix")
        .takes_value(true)
        .default_value("#/components/schemas/")
        .required(false)
        .help("the prefix for absolute $ref schema references")
}

fn make_reference_mode_arg() -> Arg<'static, 'static> {
    Arg::with_name("reference-mode")
        .short("r")
        .long("reference-mode")
        .takes_value(true)
        .possible_values(&["ref", "inline"])
        .default_value("ref")
        .required(false)
        .help("encode references as as $ref schema links or inline")
}

fn make_include_package_id_arg() -> Arg<'static, 'static> {
    Arg::with_name("include-package-id")
        .long("include-package-id")
        .required(false)
        .help("include the package id in fully qualified templates")
}

fn make_include_archive_choice_arg() -> Arg<'static, 'static> {
    Arg::with_name("include-archive-choice")
        .long("include-archive-choice")
        .required(false)
        .help("include the Archive choice which is available on every template")
}

fn make_path_style_arg() -> Arg<'static, 'static> {
    Arg::with_name("path-style")
        .short("s")
        .long("path-style")
        .takes_value(true)
        .possible_values(&["fragment", "slash"])
        .default_value("fragment")
        .required(false)
        .help("encode paths with fragment (i.e. '#') or slash ('/')")
}

fn parse_config<'c>(matches: &'c ArgMatches<'_>) -> Config<'c> {
    let dar_file = matches.value_of("dar").unwrap().to_string();
    let companion_file = matches.value_of("companion-file").unwrap().to_string();
    let format = match matches.value_of("format") {
        None => OutputFormat::Json,
        Some(s) if s == "json" => OutputFormat::Json,
        Some(s) if s == "yaml" => OutputFormat::Yaml,
        Some(s) => panic!("unknown format {}", s),
    };
    let output_file = matches.value_of("output").map(ToString::to_string);
    let module_path = matches.value_of("module-path").map(|v| v.split('.').collect::<Vec<_>>()).unwrap_or_default();
    let render_title = match matches.value_of("describe-types") {
        None => RenderTitle::None,
        Some(s) if s == "none" => RenderTitle::None,
        Some(s) if s == "data" => RenderTitle::Data,
        Some(s) if s == "all" => RenderTitle::All,
        Some(s) => panic!("unknown describe-types {}", s),
    };
    let reference_prefix = matches.value_of("reference-prefix").unwrap();
    let reference_mode = match matches.value_of("reference-mode") {
        None => ReferenceMode::default(),
        Some(s) if s == "ref" => ReferenceMode::Reference {
            prefix: reference_prefix.to_string(),
        },
        Some(s) if s == "inline" => ReferenceMode::Inline,
        Some(s) => panic!("unknown reference-prefix {}", s),
    };
    let emit_package_id = matches.is_present("include-package-id");
    let include_archive_choice = matches.is_present("include-archive-choice");
    let path_style = match matches.value_of("path-style") {
        None => PathStyle::default(),
        Some(s) if s == "fragment" => PathStyle::Fragment,
        Some(s) if s == "slash" => PathStyle::Slash,
        Some(s) => panic!("unknown path-style {}", s),
    };

    Config {
        dar_file,
        companion_file,
        format,
        output_file,
        module_path,
        render_title,
        reference_prefix,
        reference_mode,
        emit_package_id,
        include_archive_choice,
        path_style,
    }
}

/// OAS

fn execute_oas(config: &Config<'_>) -> Result<()> {
    let dar = DarFile::from_file(&config.dar_file).context(format!("dar file not found: {}", &config.dar_file))?;
    let companion_data = get_companion_data(&config.companion_file)?;
    let oas = generate_openapi(&dar, config, &companion_data)?;
    write_document(&render(&oas, config.format)?, config.output_file.as_deref())
}

fn generate_openapi(dar_file: &DarFile, config: &Config<'_>, companion_data: &CompanionData) -> Result<OpenAPI> {
    let encoder_config =
        SchemaEncoderConfig::new(RenderSchema::None, config.render_title, config.reference_mode.clone());
    dar_file.apply(|archive| {
        let encoder = JsonSchemaEncoder::new_with_config(archive, encoder_config);
        let generator = OpenAPIEncoder::new(
            archive,
            &config.module_path,
            config.reference_prefix,
            config.emit_package_id,
            config.include_archive_choice,
            config.path_style,
            companion_data,
            encoder,
        );
        generator.encode_archive()
    })?
}

/// A2S

fn execute_a2s(config: &Config<'_>) -> Result<()> {
    let dar = DarFile::from_file(&config.dar_file).context(format!("dar file not found: {}", &config.dar_file))?;
    let companion_data = get_companion_data(&config.companion_file)?;
    let a2s = generate_asyncapi(&dar, config, &companion_data)?;
    write_document(&render(&a2s, config.format)?, config.output_file.as_deref())
}

fn generate_asyncapi(dar_file: &DarFile, config: &Config<'_>, companion_data: &CompanionData) -> Result<AsyncAPI> {
    let encoder_config =
        SchemaEncoderConfig::new(RenderSchema::None, config.render_title, config.reference_mode.clone());
    dar_file.apply(|archive| {
        let encoder = JsonSchemaEncoder::new_with_config(archive, encoder_config);
        let generator = AsyncAPIEncoder::new(
            archive,
            &config.module_path,
            config.reference_prefix,
            config.emit_package_id,
            companion_data,
            encoder,
        );
        generator.encode_archive()
    })?
}

/// Common

fn get_companion_data(companion_file_name: impl AsRef<Path>) -> Result<CompanionData> {
    let path = companion_file_name.as_ref();
    if path.exists() {
        let f = std::fs::File::open(path)?;
        Ok(serde_yaml::from_reader(f)?)
    } else {
        Ok(CompanionData::default())
    }
}

fn render<S: Serialize>(oas: &S, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&oas)?),
        OutputFormat::Yaml => {
            let json_string = serde_json::to_string(&oas)?;
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(&json_string)?;
            Ok(serde_yaml::to_string(&yaml_value)?)
        },
    }
}

fn write_document(oas: &str, path: Option<&str>) -> Result<()> {
    if let Some(path) = path {
        let target = PathBuf::from(path);
        let mut file = File::create(target)?;
        Ok(file.write_all(oas.as_bytes())?)
    } else {
        Ok(io::stdout().write_all(oas.as_bytes())?)
    }
}
