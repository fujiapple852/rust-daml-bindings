//! [Daml](https://daml.com/) - The smart contract language for building distributed applications on a safe, privacy-aware runtime.
//!
//! # Crates
//!
//! The following crates are provided for working with Daml in Rust:
//!
//! | crate          | description                                 | status      |
//! |----------------|---------------------------------------------|-------------|
//! | [daml]         | Daml prelude & common entry point           | alpha       |
//! | [daml_grpc]    | Daml Ledger GRPC API bindings               | beta        |
//! | [daml_json]    | Daml Ledger JSON API bindings               | alpha       |
//! | [daml_codegen] | Rust codegen for Daml archives              | beta        |
//! | [daml_derive]  | Custom attributes for Rust<>Daml conversion | beta        |
//! | [daml_macro]   | Macros to create and extract Daml value     | beta        |
//! | [daml_util]    | Utilities to aid working with Daml ledgers  | alpha       |
//! | [daml_lf]      | Read Dar and Dalf files & bytes             | beta        |
//!
//! [daml]: index.html
//! [daml_grpc]: ../daml_grpc/index.html
//! [daml_json]: ../daml_json/index.html
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

/// The Daml prelude.
///
/// Include the Daml prelude to bring into scope all types required by the [custom attributes].  Include the prelude
/// as follows:
///
/// ```no_rust
/// use daml::prelude::*;
/// ```
/// [custom attributes]: ../../daml_derive/index.html
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
