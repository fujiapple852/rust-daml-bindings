#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(clippy::module_name_repetitions, clippy::use_self, clippy::must_use_candidate, clippy::missing_errors_doc)]
#![forbid(unsafe_code)]

use anyhow::Result;
use clap::{App, AppSettings, Arg, SubCommand};

pub mod command_download;
pub mod command_intern;
pub mod command_json;
pub mod command_list;
pub mod command_module;
pub mod command_outline;
pub mod command_package;
pub mod command_upload;
pub mod error;
pub mod package_common;

#[tokio::main(core_threads = 4)]
async fn main() -> Result<()> {
    let matches = App::new("darn")
        .version("0.1.0")
        .about("DAML dar tool")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("intern")
                .about("show dar package details")
                .arg(Arg::with_name("dar").help("Sets the input dar file to use").required(true).index(1))
                .arg(
                    Arg::with_name("index")
                        .short("i")
                        .long("index")
                        .takes_value(true)
                        .required(true)
                        .help("the string intern index"),
                ),
        )
        .subcommand(
            SubCommand::with_name("package")
                .about("show dar package details")
                .arg(Arg::with_name("dar").help("Sets the input dar file to use").required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("module")
                .about("show dar package module details")
                .arg(Arg::with_name("dar").help("Sets the input dar file to use").required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("outline")
                .about("display an outline of the dar")
                .arg(Arg::with_name("dar").help("Sets the input dar file to use").required(true).index(1))
                .arg(
                    Arg::with_name("package")
                        .short("p")
                        .long("package")
                        .takes_value(true)
                        .help("Filter for a specific package"),
                )
                .arg(
                    Arg::with_name("module")
                        .short("m")
                        .long("module")
                        .takes_value(true)
                        .help("Filter for a specific module"),
                ),
        )
        .subcommand(
            SubCommand::with_name("json")
                .about("render DAML LF as json")
                .arg(Arg::with_name("dar").help("Sets the input dar file to use").required(true).index(1))
                .arg(
                    Arg::with_name("package")
                        .short("p")
                        .long("package")
                        .takes_value(true)
                        .help("Filter for a specific package"),
                )
                .arg(
                    Arg::with_name("module")
                        .short("m")
                        .long("module")
                        .takes_value(true)
                        .help("Filter for a specific module"),
                ),
        )
        .subcommand(
            SubCommand::with_name("upload")
                .about("upload a dar to a DAML ledger")
                .arg(Arg::with_name("dar").help("Sets the input dar file to use").required(true).index(1))
                .arg(
                    Arg::with_name("uri")
                        .short("s")
                        .long("uri")
                        .takes_value(true)
                        .required(true)
                        .help("DAML ledger server uri (e.g. https://127.0.0.1:1234)"),
                )
                .arg(Arg::with_name("key").short("k").long("key").takes_value(true).help("DAML ledger server key")),
        )
        .subcommand(
            SubCommand::with_name("download")
                .about("download a dar from a DAML ledger")
                .arg(
                    Arg::with_name("main-package")
                        .help("The main package name (TODO id?) of the dar")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("output-dir")
                        .short("o")
                        .long("output-dir")
                        .takes_value(true)
                        .required(true)
                        .help("Dar output file"),
                )
                .arg(
                    Arg::with_name("uri")
                        .short("s")
                        .long("uri")
                        .takes_value(true)
                        .required(true)
                        .help("DAML ledger server uri (e.g. https://127.0.0.1:1234)"),
                )
                .arg(Arg::with_name("key").short("k").long("key").takes_value(true).help("DAML ledger server key")),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("list packages on a DAML ledger")
                .arg(
                    Arg::with_name("uri")
                        .short("s")
                        .long("uri")
                        .takes_value(true)
                        .required(true)
                        .help("DAML ledger server uri (e.g. https://127.0.0.1:1234)"),
                )
                .arg(Arg::with_name("key").short("k").long("key").takes_value(true).help("DAML ledger server key")),
        )
        .get_matches();

    // TODO
    // intern - print intern table
    // stats - table of count of things?
    // ledger - download / upload?
    // sandbox token gen
    // raw - spit our raw LF proto

    if let Some(inspect_matches) = matches.subcommand_matches("package") {
        let dar_path = inspect_matches.value_of("dar").unwrap();
        command_package::package(dar_path)?;
    }
    if let Some(inspect_matches) = matches.subcommand_matches("intern") {
        let dar_path = inspect_matches.value_of("dar").unwrap();
        let index = inspect_matches.value_of("index").unwrap();
        command_intern::intern_dotted(dar_path, index)?;
    }
    if let Some(inspect_matches) = matches.subcommand_matches("module") {
        let dar_path = inspect_matches.value_of("dar").unwrap();
        command_module::module(dar_path, inspect_matches.value_of("package"))?;
    }
    if let Some(inspect_matches) = matches.subcommand_matches("json") {
        let dar_path = inspect_matches.value_of("dar").unwrap();
        command_json::json(dar_path, inspect_matches.value_of("package"), inspect_matches.value_of("module"))?;
    }
    if let Some(inspect_matches) = matches.subcommand_matches("outline") {
        let dar_path = inspect_matches.value_of("dar").unwrap();
        command_outline::outline(dar_path, inspect_matches.value_of("package"), inspect_matches.value_of("module"))?;
    }
    if let Some(inspect_matches) = matches.subcommand_matches("upload") {
        let dar_path = inspect_matches.value_of("dar").unwrap();
        let uri = inspect_matches.value_of("uri").unwrap();
        let key = inspect_matches.value_of("key");
        command_upload::upload(dar_path, uri, key).await?;
    }
    if let Some(inspect_matches) = matches.subcommand_matches("download") {
        let main_package = inspect_matches.value_of("main-package").unwrap();
        let output_path = inspect_matches.value_of("output-dir").unwrap();
        let uri = inspect_matches.value_of("uri").unwrap();
        let key = inspect_matches.value_of("key");
        command_download::download(uri, output_path, key, main_package).await?;
    }
    if let Some(inspect_matches) = matches.subcommand_matches("list") {
        let uri = inspect_matches.value_of("uri").unwrap();
        let key = inspect_matches.value_of("key");
        command_list::list(uri, key).await?;
    }
    Ok(())
}
