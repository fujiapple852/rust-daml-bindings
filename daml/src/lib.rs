//! [DAML](https://daml.com/) - The smart contract language for building distributed applications on a safe, privacy-aware runtime.
//!
//! # Crates
//!
//! The following crates are provided for working with DAML in Rust:
//!
//! | crate                 | description                                                  | status      |
//! |-----------------------|--------------------------------------------------------------|-------------|
//! | [daml_ledger_api]     | Basic DAML Ledger API binding in Rust                        | alpha       |
//! | [daml_ledger_codegen] | Code generator for DAML modules                              | not started |
//! | [daml_ledger_derive]  | Custom attributes for converting between DAML and Rust types | alpha       |
//! | [daml_ledger_ffi]     | FFI wrapper for C-style integration                          | not started |
//! | [daml_ledger_macro]   | Macros to create and extract DAML value                      | alpha       |
//! | [daml_lf]             | Read and interpret Dar and Dalf files & bytes                | alpha       |
//!
//! [daml_ledger_api]: ../daml_ledger_api/index.html
//! [daml_ledger_codegen]: ../daml_ledger_codegen/index.html
//! [daml_ledger_derive]: ../daml_ledger_derive/index.html
//! [daml_ledger_ffi]: ../daml_ledger_ffi/index.html
//! [daml_ledger_macro]: ../daml_ledger_macro/index.html
//! [daml_lf]: ../daml_lf/index.html

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]

/// The DAML prelude.
///
/// Include the DAML prelude to bring into scope all types required by the [custom attributes].  Include the prelude
/// as follows:
///
/// ```no_rust
/// use daml::prelude::*;
/// ```
/// [custom attributes]: ../../daml_ledger_derive/index.html
pub mod prelude;
