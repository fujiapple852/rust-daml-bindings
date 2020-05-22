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
use clap::{App, AppSettings, Arg, SubCommand};

pub mod command_package;

#[tokio::main(core_threads = 4)]
async fn main() -> Result<()> {
    let matches = App::new("darn")
        .version("0.1.0")
        .about("DAML dar tool")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("package")
                .about("show dar package details")
                .arg(Arg::with_name("dar").help("Sets the input dar file to use").required(true).index(1)),
        )
        .get_matches();

    if let Some(inspect_matches) = matches.subcommand_matches("package") {
        let dar_path = inspect_matches.value_of("dar").unwrap();
        command_package::package(dar_path)?;
    }
    Ok(())
}
