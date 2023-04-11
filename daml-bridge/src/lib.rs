#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::module_name_repetitions,
    clippy::used_underscore_binding,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::significant_drop_in_scrutinee
)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]
#![doc(html_root_url = "https://docs.rs/daml-bridge/0.2.2")]
// importing the crate README as the rust doc breaks the link to the LICENCE file.
#![allow(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

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
