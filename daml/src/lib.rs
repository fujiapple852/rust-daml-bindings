//! [Daml](https://daml.com/) - The smart contract language for building distributed applications on a safe, privacy-aware runtime.
//!
//! # Crates
//!
//! The following crates are provided for working with Daml in Rust:
//!
//! | crate          | description                                                               |
//! |----------------|---------------------------------------------------------------------------|
//! | [daml]         | Daml prelude & common entry point                                         |
//! | [daml-grpc]    | Daml Ledger GRPC API bindings                                             |
//! | [daml-json]    | Daml Ledger JSON API bindings                                             |
//! | [daml-codegen] | Generate Rust GRPC API bindings from Daml archives                        |
//! | [daml-derive]  | Attribute macros for generating Rust GRPC API bindings from Daml archives |
//! | [daml-macro]   | Helper macros for working with Daml GRPC values                           |
//! | [daml-util]    | Utilities for working with Daml ledgers                                   |
//! | [daml-lf]      | Library for working with Daml-LF archives                                 |
//! | [daml-bridge]  | Daml JSON <> GRPC Ledger bridge                                           |
//!
//! [daml]: index.html
//! [daml-grpc]: https://docs.rs/daml-grpc
//! [daml-json]: https://docs.rs/daml-json
//! [daml-bridge]: https://docs.rs/daml-bridge
//! [daml-codegen]: https://docs.rs/daml-codegen
//! [daml-derive]: https://docs.rs/daml-derive
//! [daml-macro]: https://docs.rs/daml-macro
//! [daml-util]: https://docs.rs/daml-util
//! [daml-lf]: https://docs.rs/daml-lf
//! [daml-bridge]: https://docs.rs/daml-bridge
#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]

/// The Daml prelude.
///
/// Include the Daml prelude to bring into scope all types required by the [custom attributes].  Include the prelude
/// as follows:
///
/// ```no_rust
/// use daml::prelude::*;
/// ```
/// [custom attributes]: ../../daml-derive/index.html
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
