#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate, clippy::missing_errors_doc)]
#![forbid(unsafe_code)]

use std::convert::TryFrom;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg, ArgMatches};
use serde::Serialize;

use companion::CompanionData;
use config::PathStyle;
use config::{Config, OutputFormat};
use daml::json_api::schema_encoder::{
    DataDict, JsonSchemaEncoder, ReferenceMode, RenderDescription, RenderSchema, RenderTitle, SchemaEncoderConfig,
};
use daml::lf::DarFile;
use filter::{TemplateFilter, TemplateFilterInput};
use oas::OpenAPI;
use oas::OpenAPIEncoder;

use crate::a2s::AsyncAPI;
use crate::a2s::AsyncAPIEncoder;
use log::LevelFilter;
use serde::de::DeserializeOwned;
use simple_logger::SimpleLogger;

mod a2s;
mod choice_event_extractor;
mod common;
mod companion;
mod component_encoder;
mod config;
mod data_searcher;
mod filter;
mod format;
mod json_api_schema;
mod oas;
mod schema;
mod util;

const DEFAULT_DATA_DICT_FILE: &str = ".datadict.yaml";
const DEFAULT_TEMPLATE_FILTER_FILE: &str = ".template_filter.yaml";
const DEFAULT_COMPANION_FILE: &str = ".companion.yaml";

fn main() -> Result<()> {
    let oas = App::new("oas")
        .about("Generate an OpenAPI document from the given Dar file")
        .arg(make_dar_arg())
        .arg(make_log_level_arg())
        .arg(make_format_arg())
        .arg(make_output_arg())
        .arg(make_companion_file_arg())
        .arg(make_datadict_file_arg())
        .arg(make_template_filter_file_arg())
        .arg(make_module_path_arg())
        .arg(make_data_title_arg())
        .arg(make_type_description_arg())
        .arg(make_reference_prefix_arg())
        .arg(make_reference_mode_arg())
        .arg(make_include_package_id_arg())
        .arg(make_include_archive_choice_arg())
        .arg(make_include_general_operations_arg())
        .arg(make_path_style_arg());
    let a2s = App::new("a2s")
        .about("Generate an AsyncAPI document from the given Dar file")
        .arg(make_dar_arg())
        .arg(make_log_level_arg())
        .arg(make_format_arg())
        .arg(make_output_arg())
        .arg(make_companion_file_arg())
        .arg(make_datadict_file_arg())
        .arg(make_template_filter_file_arg())
        .arg(make_module_path_arg())
        .arg(make_data_title_arg())
        .arg(make_type_description_arg())
        .arg(make_reference_prefix_arg())
        .arg(make_reference_mode_arg())
        .arg(make_include_package_id_arg());
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .term_width(0)
        .subcommand(oas)
        .subcommand(a2s)
        .get_matches();
    match matches.subcommand() {
        Some(("oas", sub)) => execute_oas(&parse_config(sub))?,
        Some(("a2s", sub)) => execute_a2s(&parse_config(sub))?,
        _ => {},
    };
    Ok(())
}

fn make_dar_arg() -> Arg<'static> {
    Arg::new("dar").help("Sets the input dar file to use").required(true).index(1)
}

fn make_log_level_arg() -> Arg<'static> {
    Arg::new("v").required(false).short('v').multiple_occurrences(true).help("Sets the level of verbosity")
}

fn make_format_arg() -> Arg<'static> {
    Arg::new("format")
        .short('f')
        .long("format")
        .takes_value(true)
        .possible_values(&["json", "yaml"])
        .default_value("json")
        .required(false)
        .help("the output format")
}

fn make_output_arg() -> Arg<'static> {
    Arg::new("output").short('o').long("output").takes_value(true).required(false).help("the output file path")
}

fn make_companion_file_arg() -> Arg<'static> {
    Arg::new("companion-file")
        .short('c')
        .long("companion-file")
        .takes_value(true)
        .required(false)
        .help("the companion yaml file with auxiliary data to inject into the generated OAS document")
}

fn make_datadict_file_arg() -> Arg<'static> {
    Arg::new("datadict-file")
        .short('d')
        .long("datadict-file")
        .takes_value(true)
        .required(false)
        .help("the data dictionary to use to augment the generated JSON schema")
}

fn make_template_filter_file_arg() -> Arg<'static> {
    Arg::new("template-filter-file")
        .short('t')
        .long("template-filter-file")
        .takes_value(true)
        .required(false)
        .help("the template filter to apply")
}

fn make_module_path_arg() -> Arg<'static> {
    Arg::new("module-path")
        .short('m')
        .long("module")
        .takes_value(true)
        .required(false)
        .help("module path prefix in the form Foo.Bar.Baz")
}

fn make_data_title_arg() -> Arg<'static> {
    Arg::new("data-title")
        .long("data-title")
        .takes_value(true)
        .possible_values(&["none", "data"])
        .default_value("data")
        .required(false)
        .help("include the `title` property describing the data item name (i.e. Foo.Bar:Baz)")
}

fn make_type_description_arg() -> Arg<'static> {
    Arg::new("type-description")
        .long("type-description")
        .takes_value(true)
        .possible_values(&["none", "data", "all"])
        .default_value("all")
        .required(false)
        .help("include the `description` property describing the Daml type")
}

fn make_reference_prefix_arg() -> Arg<'static> {
    Arg::new("reference-prefix")
        .short('p')
        .long("reference-prefix")
        .takes_value(true)
        .default_value("#/components/schemas/")
        .required(false)
        .help("the prefix for absolute $ref schema references")
}

fn make_reference_mode_arg() -> Arg<'static> {
    Arg::new("reference-mode")
        .short('r')
        .long("reference-mode")
        .takes_value(true)
        .possible_values(&["ref", "inline"])
        .default_value("ref")
        .required(false)
        .help("encode references as as $ref schema links or inline")
}

fn make_include_package_id_arg() -> Arg<'static> {
    Arg::new("include-package-id")
        .long("include-package-id")
        .required(false)
        .help("include the package id in fully qualified templates")
}

fn make_include_archive_choice_arg() -> Arg<'static> {
    Arg::new("include-archive-choice")
        .long("include-archive-choice")
        .required(false)
        .help("include the Archive choice which is available on every template")
}

fn make_include_general_operations_arg() -> Arg<'static> {
    Arg::new("include-general-operations").long("include-general-operations").required(false).help(
        "include the general (non-template specific) /v1/create, /v1/exercise, /v1/create-and-exercise & /v1/fetch \
         endpoints",
    )
}

fn make_path_style_arg() -> Arg<'static> {
    Arg::new("path-style")
        .short('s')
        .long("path-style")
        .takes_value(true)
        .possible_values(&["fragment", "slash"])
        .default_value("fragment")
        .required(false)
        .help("encode paths with fragment (i.e. '#') or slash ('/')")
}

fn parse_config(matches: &ArgMatches) -> Config<'_> {
    let dar_file = matches.value_of("dar").unwrap().to_string();
    let level_filter = match matches.occurrences_of("v") {
        0 => LevelFilter::Off,
        1 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    let companion_file = matches.value_of("companion-file").map(ToString::to_string);
    let data_dict_file = matches.value_of("datadict-file").map(ToString::to_string);
    let template_filter_file = matches.value_of("template-filter-file").map(ToString::to_string);
    let format = match matches.value_of("format") {
        None => OutputFormat::Json,
        Some(s) if s == "json" => OutputFormat::Json,
        Some(s) if s == "yaml" => OutputFormat::Yaml,
        Some(s) => panic!("unknown format {}", s),
    };
    let output_file = matches.value_of("output").map(ToString::to_string);
    let module_path = matches.value_of("module-path").map(|v| v.split('.').collect::<Vec<_>>()).unwrap_or_default();
    let render_title = match matches.value_of("data-title") {
        None => RenderTitle::None,
        Some(s) if s == "none" => RenderTitle::None,
        Some(s) if s == "data" => RenderTitle::Data,
        Some(s) => panic!("unknown data-title {}", s),
    };
    let render_description = match matches.value_of("type-description") {
        None => RenderDescription::None,
        Some(s) if s == "none" => RenderDescription::None,
        Some(s) if s == "data" => RenderDescription::Data,
        Some(s) if s == "all" => RenderDescription::All,
        Some(s) => panic!("unknown type-description {}", s),
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

    let include_general_operations = matches.is_present("include-general-operations");

    let path_style = match matches.value_of("path-style") {
        None => PathStyle::default(),
        Some(s) if s == "fragment" => PathStyle::Fragment,
        Some(s) if s == "slash" => PathStyle::Slash,
        Some(s) => panic!("unknown path-style {}", s),
    };

    Config {
        dar_file,
        companion_file,
        data_dict_file,
        template_filter_file,
        format,
        output_file,
        module_path,
        render_title,
        render_description,
        reference_prefix,
        reference_mode,
        emit_package_id,
        include_archive_choice,
        include_general_operations,
        path_style,
        level_filter,
    }
}

/// OAS

fn execute_oas(config: &Config<'_>) -> Result<()> {
    SimpleLogger::new().with_level(config.level_filter).init().unwrap();
    log::info!("Generating OAS specification documents for {}", config.dar_file);
    log::info!("Loading dar file...");
    let dar = DarFile::from_file(&config.dar_file).context(format!("dar file not found: {}", &config.dar_file))?;
    log::info!("Loading companion data file...");
    let companion_data = get_companion_data(&config.companion_file)?;
    log::info!("Loading data dict file...");
    let data_dict = get_data_dict(&config.data_dict_file)?;
    log::info!("Loading template filter file...");
    let template_filter = get_template_filter(&config.template_filter_file)?;
    log::info!("Generating API document...");
    let oas = generate_openapi(&dar, config, &companion_data, data_dict, &template_filter)?;
    log::info!("Rendering API document...");
    let rendered = render(&oas, config.format)?;
    log::info!("Writing API document...");
    write_document(&rendered, config.output_file.as_deref())
}

fn generate_openapi(
    dar_file: &DarFile,
    config: &Config<'_>,
    companion_data: &CompanionData,
    data_dict: DataDict,
    template_filter: &TemplateFilter,
) -> Result<OpenAPI> {
    let encoder_config = SchemaEncoderConfig::new(
        RenderSchema::None,
        config.render_title,
        config.render_description,
        config.reference_mode.clone(),
        data_dict,
    );
    dar_file.apply(|archive| {
        let encoder = JsonSchemaEncoder::new_with_config(archive, encoder_config);
        let generator = OpenAPIEncoder::new(
            archive,
            &config.module_path,
            template_filter,
            config.reference_prefix,
            config.emit_package_id,
            config.include_archive_choice,
            config.include_general_operations,
            config.path_style,
            companion_data,
            encoder,
        );
        generator.encode_archive()
    })?
}

/// A2S

fn execute_a2s(config: &Config<'_>) -> Result<()> {
    SimpleLogger::new().with_level(config.level_filter).init().unwrap();
    log::info!("Generating A2S specification documents for {}", config.dar_file);
    let dar = DarFile::from_file(&config.dar_file).context(format!("dar file not found: {}", &config.dar_file))?;
    let companion_data = get_companion_data(&config.companion_file)?;
    let data_dict = get_data_dict(&config.data_dict_file)?;
    let template_filter = get_template_filter(&config.template_filter_file)?;
    let a2s = generate_asyncapi(&dar, config, &companion_data, data_dict, &template_filter)?;
    write_document(&render(&a2s, config.format)?, config.output_file.as_deref())
}

fn generate_asyncapi(
    dar_file: &DarFile,
    config: &Config<'_>,
    companion_data: &CompanionData,
    data_dict: DataDict,
    template_filter: &TemplateFilter,
) -> Result<AsyncAPI> {
    let encoder_config = SchemaEncoderConfig::new(
        RenderSchema::None,
        config.render_title,
        config.render_description,
        config.reference_mode.clone(),
        data_dict,
    );
    dar_file.apply(|archive| {
        let encoder = JsonSchemaEncoder::new_with_config(archive, encoder_config);
        let generator = AsyncAPIEncoder::new(
            archive,
            &config.module_path,
            template_filter,
            config.reference_prefix,
            config.emit_package_id,
            companion_data,
            encoder,
        );
        generator.encode_archive()
    })?
}

/// Common

fn get_companion_data(filter_file_name: &Option<String>) -> Result<CompanionData> {
    read_file(filter_file_name, DEFAULT_COMPANION_FILE)
        .map_err(|err| anyhow!("failed to parse companion file").context(err))
}

fn get_data_dict(data_dict_file_name: &Option<String>) -> Result<DataDict> {
    read_file(data_dict_file_name, DEFAULT_DATA_DICT_FILE)
        .map_err(|err| anyhow!("failed to parse datadict file").context(err))
}

fn get_template_filter(filter_file_name: &Option<String>) -> Result<TemplateFilter> {
    let filter: TemplateFilterInput = read_file(filter_file_name, DEFAULT_TEMPLATE_FILTER_FILE)?;
    TemplateFilter::try_from(filter).map_err(|err| anyhow!("failed to parse template filter file").context(err))
}

fn read_file<T: DeserializeOwned + Default, S: AsRef<str>>(file_name: &Option<String>, fallback: S) -> Result<T> {
    let path = file_name.as_ref();
    if let Some(name) = path {
        let path = PathBuf::from(name);
        if path.is_file() && path.exists() {
            let f = std::fs::File::open(path)?;
            Ok(serde_yaml::from_reader(f).map_err(|err| anyhow!("failed to parse file {}", name).context(err))?)
        } else {
            Err(anyhow!(format!("file {} not found", path.display())))
        }
    } else {
        let path = PathBuf::from(fallback.as_ref());
        if path.is_file() && path.exists() {
            let f = std::fs::File::open(path)?;
            Ok(serde_yaml::from_reader(f)
                .map_err(|err| anyhow!("failed to parse file {}", fallback.as_ref()).context(err))?)
        } else {
            Ok(T::default())
        }
    }
}

fn render<S: Serialize>(doc: &S, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&doc)?),
        OutputFormat::Yaml => {
            let json_string = serde_json::to_string(&doc)?;
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(&json_string)?;
            Ok(serde_yaml::to_string(&yaml_value)?)
        },
    }
}

fn write_document(doc: &str, path: Option<&str>) -> Result<()> {
    if let Some(path) = path {
        let target = PathBuf::from(path);
        let mut file = File::create(target)?;
        Ok(file.write_all(doc.as_bytes())?)
    } else {
        Ok(io::stdout().write_all(doc.as_bytes())?)
    }
}
