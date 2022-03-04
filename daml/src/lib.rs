//! [Daml](https://daml.com/) - The smart contract language for building distributed applications on a safe, privacy-aware runtime.
//!
//! # Library Crates
//!
//! The following library crates are provided for working with Daml in Rust:
//!
//! | crate                          | description                                                               |
//! |--------------------------------|---------------------------------------------------------------------------|
//! | [daml](self)                   | Daml prelude & common entry point                                         |
//! | [daml-grpc](::daml_grpc)       | Daml Ledger GRPC API bindings                                             |
//! | [daml-json](::daml_json)       | Daml Ledger JSON API bindings                                             |
//! | [daml-codegen](::daml_codegen) | Library for generate Rust GRPC API bindings from Daml archives            |
//! | [daml-derive](::daml_derive)   | Attribute macros for generating Rust GRPC API bindings from Daml archives |
//! | [daml-macro](::daml_macro)     | Helper macros for working with Daml GRPC values                           |
//! | [daml-util](::daml_util)       | Utilities for working with Daml ledgers                                   |
//! | [daml-lf](::daml_lf)           | Library for working with Daml-LF archives                                 |
//! | [daml-bridge]                  | Library for Daml JSON <> GRPC Ledger bridging                             |
//!
//! # Tools
//!
//! The following tools are provided:
//!
//! | crate                          | description                                                               |
//! |--------------------------------|---------------------------------------------------------------------------|
//! | [daml-codegen](::daml_codegen) | generate Rust GRPC API bindings from Daml archives                        |
//! | [daml-bridge]                  | Daml JSON <> GRPC Ledger bridge                                           |
//! | [daml2oas]                     | Generate OpenAPI and AsyncAPI specifications from Daml dar files          |
//! | [daml-darn]                    | Daml Archive cli tool                                                     |
//!
//! # Examples
//!
//! Further examples are available in the [`examples`](https://github.com/fujiapple852/rust-daml-bindings/tree/master/examples) directory.
//!
//! [daml-bridge]: https://docs.rs/daml-bridge/0.1.1/daml_bridge
//! [daml2oas]: https://docs.rs/daml2oas/0.1.1/daml2oas
//! [daml-darn]: https://docs.rs/daml-darn/0.1.1/daml-darn

#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]
#![doc(html_root_url = "https://docs.rs/daml/0.1.1")]

/// The Daml prelude.
///
/// Include the Daml prelude to bring into scope all types required by the [custom attributes](::daml_derive).  Include
/// the prelude as follows:
///
/// ```no_rust
/// use daml::prelude::*;
/// ```
#[cfg(feature = "prelude")]
pub mod prelude;

#[cfg(feature = "grpc")]
#[doc(hidden)]
pub mod grpc_api {
    pub use daml_grpc::*;
}

#[cfg(feature = "json")]
#[doc(hidden)]
pub mod json_api {
    pub use daml_json::*;
}

#[cfg(feature = "codegen")]
#[doc(hidden)]
pub mod codegen {
    pub use daml_codegen::*;
}

#[cfg(feature = "derive")]
#[doc(hidden)]
pub mod derive {
    pub use daml_derive::*;
}

#[cfg(feature = "lf")]
#[doc(hidden)]
pub mod lf {
    pub use daml_lf::*;
}

#[cfg(feature = "util")]
#[doc(hidden)]
pub mod util {
    pub use daml_util::*;
}

#[cfg(feature = "macros")]
#[doc(hidden)]
pub mod macros {
    pub use daml_macro::*;
}
