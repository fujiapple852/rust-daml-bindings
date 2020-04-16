//! [DAML](https://daml.com/) - The smart contract language for building distributed applications on a safe, privacy-aware runtime.
//!
//! # Crates
//!
//! The following crates are provided for working with DAML in Rust:
//!
//! | crate          | description                                                  | status      |
//! |----------------|--------------------------------------------------------------|-------------|
//! | [daml]         | DAML prelude & common entry point                            | alpha       |
//! | [daml_api]     | Basic DAML Ledger API binding in Rust                        | alpha       |
//! | [daml_codegen] | Rust code generator for DAML archives                        | alpha       |
//! | [daml_derive]  | Procedural macros for converting between DAML and Rust types | alpha       |
//! | [daml_macro]   | Macros to create and extract DAML value                      | alpha       |
//! | [daml_util]    | Utilities to aid working with DAML ledgers                   | alpha       |
//! | [daml_lf]      | Read and interpret Dar and Dalf files & bytes                | alpha       |
//!
//! [daml_api]: ../daml_api/index.html
//! [daml_codegen]: ../daml_codegen/index.html
//! [daml_derive]: ../daml_derive/index.html
//! [daml_macro]: ../daml_macro/index.html
//! [daml_util]: ../daml_util/index.html
//! [daml_lf]: ../daml_lf/index.html
#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]
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
/// [custom attributes]: ../../daml_derive/index.html
#[cfg(feature = "prelude")]
pub mod prelude;

///
#[cfg(feature = "api")]
pub mod api {
    pub use daml_api::*;
}

///
#[cfg(feature = "codegen")]
pub mod codegen {
    pub use daml_codegen::*;
}

///
#[cfg(feature = "derive")]
pub mod derive {
    pub use daml_derive::*;
}

/// DAML LF (ledger fragment).
#[cfg(feature = "lf")]
pub mod lf {
    pub use daml_lf::*;
}

///
#[cfg(feature = "util")]
pub mod util {
    pub use daml_util::*;
}

///
#[cfg(feature = "macros")]
pub mod macros {
    pub use daml_macro::*;
}
