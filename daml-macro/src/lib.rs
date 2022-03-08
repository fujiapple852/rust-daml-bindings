//! Helper macros for working with the [Daml GRPC Ledger API](::daml_grpc)
//!
//! Provides a [`daml_value!`] macro to simplify the construction of [`DamlValue`](daml_grpc::data::value::DamlValue)
//! literals and a [`daml_path!`] macro to simplify the extraction of data from existing
//! [`DamlRecord`](daml_grpc::data::value::DamlRecord) & [`DamlValue`](daml_grpc::data::value::DamlValue) literals.

#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(clippy::module_name_repetitions, clippy::shadow_unrelated, clippy::unit_cmp)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]
#![doc(html_root_url = "https://docs.rs/daml-macro/0.2.2")]

mod path;
mod value;

// Reexport crates as the macros use several types they define.
// TODO should reference type aliases here rather than raw chrono / bigdecimal but requires some rework in the macros
#[doc(hidden)]
pub use bigdecimal;
#[doc(hidden)]
pub use chrono;
#[doc(hidden)]
pub use daml_grpc;

#[cfg(test)]
mod test_util;
