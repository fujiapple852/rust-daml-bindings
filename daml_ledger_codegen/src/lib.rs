//! Rust DAML type code generator.
//!
//! Provides an implementation of custom attributes and procedural macros defined in the [`daml_ledger_derive`] crate.
//!
//! [`daml_ledger_derive`]: ../daml_ledger_derive/index.html

#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::default_trait_access,
    clippy::needless_pass_by_value,
    clippy::use_self,
    clippy::lippy::cast_sign_loss
)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]
#![recursion_limit = "128"]

extern crate proc_macro;

/// Representation of DAML types.
mod element;

/// Converters from DAML types to Rust types.
mod convert;

/// Element renderers.
mod renderer;

/// Code generators for producing Rust implementations of DAML types.
pub mod generator;
