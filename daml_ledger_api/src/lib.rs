//! A DAML Ledger provides an [API](https://docs.daml.com/app-dev/index.html) to receive data
//! from and send data to the ledger.
//!
//! The API is separated into a small number of services that cover various aspects of the ledger, e.g. reading
//! transactions or submitting commands.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]

/// DAML API domain objects (i.e. values, commands, events).
pub mod data;

/// DAML GRPC API services (i.e. command & transaction services).
pub mod service;

mod ledger_client;
pub use ledger_client::DamlLedgerClient;

mod command_factory;
pub use command_factory::DamlCommandFactory;

mod grpc_protobuf_autogen;
mod util;

// Re-export types used by the public API
#[doc(hidden)]
pub use bigdecimal;
#[doc(hidden)]
pub use chrono;
