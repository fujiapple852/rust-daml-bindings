//! Daml JSON <> GRPC Bridge.
//!
//! Provides a library and executable for bridging the JSON and GRPC Daml ledger APIs.

#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::module_name_repetitions,
    clippy::used_underscore_binding,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value
)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]

mod aliases;
mod bridge;
mod config;
mod server;
mod handler {
    mod common;
    pub mod create_and_exercise_handler;
    pub mod create_handler;
    pub mod exercise_by_key_handler;
    pub mod exercise_handler;
    pub mod packages_handler;
    pub mod parties_handler;
}

pub use bridge::Bridge;
pub use config::BridgeConfigData;