//! Helper macros for working with the [DAML Ledger API]
//!
//! Provides a [`daml_value!`] macro to simplify the construction of [`DamlValue`] literals and a
//! [`daml_path!`] macro to simplify the extraction of data from existing [`DamlRecord`]  &
//! [`DamlValue`] literals.
//!
//! [`DamlValue`]: ../../doc/daml_api/data/value/enum.DamlValue.html
//! [`DamlRecord`]: ../../doc/daml_api/data/value/struct.DamlRecord.html
//! [DAML Ledger API]: ../../doc/daml_api/index.html
//! [`daml_value!`]: macro.daml_value.html
//! [`daml_path!`]: macro.daml_path.html

#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(clippy::module_name_repetitions, clippy::shadow_unrelated, clippy::unit_cmp)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]

mod path;
mod value;

#[cfg(test)]
mod test_util;
