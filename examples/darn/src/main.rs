#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::module_name_repetitions,
    clippy::use_self,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::cast_sign_loss
)]
#![forbid(unsafe_code)]

use crate::command_intern::CommandIntern;
use crate::command_package::CommandPackage;
use crate::command_token::CommandToken;
use anyhow::Result;
use clap::{crate_description, crate_name, crate_version, App, AppSettings, ArgMatches};
use std::collections::HashMap;

pub mod command_intern;
pub mod command_package;
pub mod command_token;

pub trait DarnCommand {
    fn name(&self) -> &str;
    fn args<'a>(&self) -> App<'a>;
    fn execute(&self, matches: &ArgMatches) -> Result<()>;
}

macro_rules! command {
    ($id:ident) => {
        Box::new($id {})
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    let commands: Vec<Box<dyn DarnCommand>> =
        vec![command!(CommandPackage), command!(CommandToken), command!(CommandIntern)];
    let command_map: HashMap<_, _> = commands.into_iter().map(|cmd| (cmd.name().to_owned(), cmd)).collect();
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommands(command_map.values().map(|cmd| cmd.args()))
        .get_matches();
    let (sub, args) = matches.subcommand().unwrap();
    command_map[sub].execute(args)?;
    Ok(())
}
