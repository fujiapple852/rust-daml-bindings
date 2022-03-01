//! [Daml](https://daml.com/) - The smart contract language for building distributed applications on a safe, privacy-aware runtime.
//!
//! # Crates
//!
//! The following crates are provided for working with Daml in Rust:
//!
//! | crate          | description                                 | status      |
//! |----------------|---------------------------------------------|-------------|
//! | [daml]         | Daml prelude & common entry point           | alpha       |
//! | [daml-grpc]    | Daml Ledger GRPC API bindings               | beta        |
//! | [daml-json]    | Daml Ledger JSON API bindings               | alpha       |
//! | [daml-bridge]  | Daml JSON<>GRPC Ledger bridge               | alpha       |
//! | [daml-codegen] | Rust codegen for Daml archives              | beta        |
//! | [daml-derive]  | Custom attributes for Rust<>Daml conversion | beta        |
//! | [daml-macro]   | Macros to create and extract Daml value     | beta        |
//! | [daml-util]    | Utilities to aid working with Daml ledgers  | alpha       |
//! | [daml-lf]      | Read Dar and Dalf files & bytes             | beta        |
//!
//! [daml]: index.html
//! [daml-grpc]: ../daml-grpc/index.html
//! [daml-json]: ../daml-json/index.html
//! [daml-bridge]: ../daml-bridge/index.html
//! [daml-codegen]: ../daml-codegen/index.html
//! [daml-derive]: ../daml-derive/index.html
//! [daml-macro]: ../daml-macro/index.html
//! [daml-util]: ../daml-util/index.html
//! [daml-lf]: ../daml-lf/index.html
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
