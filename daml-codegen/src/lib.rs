//! Rust Daml type code generator.
//!
//! Provides an implementation of custom attributes and procedural macros defined in the [`daml-derive`] crate.
//!
//! [`daml-derive`]: https://docs.rs/daml-derive

#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::use_self,
    clippy::cast_sign_loss,
    clippy::must_use_candidate,
    clippy::missing_errors_doc
)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]
#![doc(html_root_url = "https://docs.rs/daml-codegen/0.1.1")]
#![recursion_limit = "128"]

mod error;

/// Element renderers.
pub mod renderer;

/// Code generators for producing Rust implementations of Daml types.
pub mod generator;
