#![doc = include_str!("../README.md")]
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
use clap::{crate_description, crate_name, crate_version, ArgMatches, Command};
use std::collections::HashMap;

#[doc(hidden)]
pub mod command_intern;
#[doc(hidden)]
pub mod command_package;
#[doc(hidden)]
pub mod command_token;

#[doc(hidden)]
pub trait DarnCommand {
    fn name(&self) -> &str;
    fn args<'a>(&self) -> Command<'a>;
    fn execute(&self, matches: &ArgMatches) -> Result<()>;
}

#[doc(hidden)]
macro_rules! command {
    ($id:ident) => {
        Box::new($id {})
    };
}

#[doc(hidden)]
#[tokio::main]
async fn main() -> Result<()> {
    let commands: Vec<Box<dyn DarnCommand>> =
        vec![command!(CommandPackage), command!(CommandToken), command!(CommandIntern)];
    let command_map: HashMap<_, _> = commands.into_iter().map(|cmd| (cmd.name().to_owned(), cmd)).collect();
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg_required_else_help(true)
        .subcommands(command_map.values().map(|cmd| cmd.args()))
        .get_matches();
    let (sub, args) = matches.subcommand().unwrap();
    command_map[sub].execute(args)?;
    Ok(())
}
