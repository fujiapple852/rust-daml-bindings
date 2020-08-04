//! DAML ledger GRPC [API](https://docs.daml.com/app-dev/index.html).
//!
//! The API is separated into a small number of services that cover various aspects of the ledger, e.g. reading
//! transactions or submitting commands.

#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::module_name_repetitions,
    clippy::use_self,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_const_for_fn,
    clippy::used_underscore_binding,
    clippy::future_not_send
)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]

/// DAML API domain objects (i.e. values, commands, events).
pub mod data;

/// DAML GRPC API services (i.e. command & transaction services).
pub mod service;

/// DAML primitive data types.
pub mod primitive_types;

/// Serialize & Deserialize DAML types.
pub mod serialize;

/// Nat types for specifying DAML Numeric types.
pub mod nat;

mod ledger_client;
pub use ledger_client::{DamlGrpcClient, DamlGrpcClientBuilder};

mod command_factory;
pub use command_factory::DamlCommandFactory;

mod executor;
pub use executor::{CommandExecutor, DamlSimpleExecutor, DamlSimpleExecutorBuilder, Executor};

mod grpc_protobuf;
mod util;