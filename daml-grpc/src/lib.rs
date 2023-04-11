//! Daml ledger GRPC [API](https://docs.daml.com/app-dev/grpc/index.html).
//!
//! This library provides a client for the Daml ledger GRPC API.
//!
//! # Example
//!
//! The following example demonstrates creating a [`DamlGrpcClient`] using the [`DamlGrpcClientBuilder`], then creating
//! a [`DamlSimpleExecutor`] using the [`DamlSimpleExecutorBuilder`] and finally creating and submitting a
//! [`DamlCreateCommand`](data::command::DamlCreateCommand) to the ledger:
//!
//! ```no_run
//! # use futures::future::Future;
//! # use daml_grpc::data::command::DamlCommand;
//! # use daml_grpc::DamlGrpcClientBuilder;
//! # use daml_grpc::DamlSimpleExecutorBuilder;
//! # use daml_grpc::data::DamlResult;
//! # use daml_grpc::CommandExecutor;
//! # use daml_grpc::data::command::DamlCreateCommand;
//! # use daml_grpc::data::DamlIdentifier;
//! # use daml_grpc::data::value::DamlRecord;
//! # use std::error::Error;
//! # fn main() -> DamlResult<()> {
//! # futures::executor::block_on(async {
//! let client = DamlGrpcClientBuilder::uri("http://localhost:8082").connect().await?;
//! let executor = DamlSimpleExecutorBuilder::new(&client).act_as("Alice").build()?;
//! let template_id = DamlIdentifier::new("...", "Fuji.PingPong", "Ping");
//! let record = DamlRecord::new(vec![], None::<DamlIdentifier>);
//! let command = DamlCreateCommand::new(template_id, record);
//! let create_event = executor.execute_create(command).await?;
//! # Ok(())
//! # })
//! # }
//! ```
//!
//! Note that Daml commands such as [`DamlCreateCommand`](data::command::DamlCreateCommand) can be automatically
//! generated for existing Daml templates using the various functions and macros provided in the [`daml-codegen`](https://docs.rs/daml-codegen/0.2.2) crate.
//!
//! Note also that the [`daml_value`](https://docs.rs/daml-macro/0.2.2/daml_macro/macro.daml_value.html) macro is
//! provided to simplify the construction of [`DamlRecord`](data::value::DamlRecord) and
//! [`DamlValue`](data::value::DamlValue) types.

#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::module_name_repetitions,
    clippy::use_self,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_const_for_fn,
    clippy::used_underscore_binding,
    clippy::future_not_send,
    clippy::return_self_not_must_use,
    clippy::option_if_let_else
)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]
#![doc(html_root_url = "https://docs.rs/daml-grpc/0.2.2")]

/// Daml API domain objects (i.e. values, commands, events).
pub mod data;

/// Daml GRPC API services (i.e. command & transaction services).
pub mod service;

/// Daml primitive data types.
pub mod primitive_types;

/// Serialize & Deserialize Daml types.
pub mod serialize;

/// Nat types for specifying Daml Numeric types.
pub mod nat;

mod ledger_client;
pub use ledger_client::{DamlGrpcClient, DamlGrpcClientBuilder};

mod command_factory;
pub use command_factory::DamlCommandFactory;

mod executor;
pub use executor::{CommandExecutor, DamlSimpleExecutor, DamlSimpleExecutorBuilder, Executor};

mod grpc_protobuf;
mod util;
