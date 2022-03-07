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
//! # Features
//!
//! The following feature may be enabled:
//!
//! - `grpc` - enable the `daml-grpc` library
//! - `json` - enable the `daml-json` library
//! - `codegen` - enable the `daml-codegen` library
//! - `derive` - enable the `daml-derive` library
//! - `macros` - enable the `daml-macros` library
//! - `util` - enable the `daml-util` library
//! - `lf` - enable the `daml-lf` library (excludes expressions)
//! - `lf-full` - enable the `daml-lf` library (includes expressions)
//! - `prelude` - enable the `daml` prelude
//! - `full` - enables: `grpc`, `json`, `macros`, `derive`, `codegen`, `lf-full`, `util`, `prelude`
//! - `sandbox` - enable sandbox testing features
//!
//! # Usage
//!
//! Applications should always depend on the `daml` crate directly and specify the appropriate features to enable the
//! required functionality:
//!
//! ```toml
//! [dependencies]
//! daml = { version = "0.2.0", features = [ "full" ] }
//! ```
//!
//! # Tour
//!
//! The following is a brief tour of the facilities these libraries provide.
//!
//! ## Working with GRPC
//!
//! To connect to a Daml ledger via GRPC you can use the [`DamlGrpcClient`](daml_grpc::DamlGrpcClient) which can be
//! created using the [`DamlGrpcClientBuilder`](daml_grpc::DamlGrpcClientBuilder).  This client exposes all Daml GRPC
//! [`service`](daml_grpc::service)s, such as the [`DamlCommandService`](daml_grpc::service::DamlCommandService).  All
//! services make use of [`DamlValue`](daml_grpc::data::value::DamlValue) which is generic GRPC representation of Daml
//! types.
//!
//! The [`DamlSimpleExecutor`](daml_grpc::DamlSimpleExecutor) which can be created using the
//! [`DamlSimpleExecutorBuilder`](daml_grpc::DamlSimpleExecutorBuilder) and provides a higher level API for creating an
//! executing commands against the GRPC Daml ledger API.
//!
//! A [`DamlSandboxTokenBuilder`](daml_util::DamlSandboxTokenBuilder) is provided for constructing `JWT` tokens suitable
//! for use with the Daml sandbox and other ledgers.
//!
//! The [`daml_value!`](daml_macro::daml_value) macro can be used to simplify creating a
//! [`DamlValue`](daml_grpc::data::value::DamlValue) and the [`daml_path!`](daml_macro::daml_path) macro can be used to
//! extract data values from a [`DamlValue`](daml_grpc::data::value::DamlValue).
//!
//! A sample application which uses many of these is facilities available in the
//! [`grpc-demo` example](https://github.com/fujiapple852/rust-daml-bindings/tree/master/examples/grpc-demo).  See also
//! the [`integration_tests`](https://github.com/fujiapple852/rust-daml-bindings/tree/master/daml-grpc/tests/grpc) in
//! the [`daml-grpc`](::daml_grpc) crate for comprehensive usage examples.
//!
//! ## Working with JSON
//!
//! To connect to a Daml ledger via the JSON API you can use the [DamlJsonClient](daml_json::service::DamlJsonClient)
//! which can be created using the [DamlJsonClientBuilder](daml_json::service::DamlJsonClientBuilder).  This client
//! exposes the full Daml JSON API [`service`](daml_json::service) which make use of the generic JSON `Value` type.
//!
//! Conversion between the generic GRPC [`DamlValue`](`daml_grpc::data::value::DamlValue`) and the generic JSON
//! `Value` representations is provided by [JsonValueEncoder](daml_json::value_encode::JsonValueEncoder) and
//! [JsonValueDecoder](daml_json::value_decode::JsonValueDecoder).
//!
//! It is also possible to convert A Daml JSON API [`request`](daml_json::request) to GRPC API
//! [`command`](daml_grpc::data::command) using
//! [JsonToGrpcRequestConverter](daml_json::request_converter::JsonToGrpcRequestConverter) and A Daml GRPC API
//! [`event`](daml_grpc::data::event) to a JSON API [`response`](daml_json::request) using
//! [GrpcToJsonResponseConverter](daml_json::response_converter::GrpcToJsonResponseConverter).
//!
//! A [JsonSchemaEncoder](daml_json::schema_encoder::JsonSchemaEncoder) is provided to generate JSON Schema documents
//! from Daml elements and archives.
//!
//! See the
//! [`integration_tests`](https://github.com/fujiapple852/rust-daml-bindings/blob/master/daml-json/tests/json/all_json_api_tests.rs)
//! in the [`daml_json`](::daml_json) crate for comprehensive usage examples.
//!
//! ## Working with Daml LF
//!
//! The [`DarFile`](daml_lf::DarFile) and [`DamlLfArchive`](daml_lf::DamlLfArchive) types can be used to load and
//! parse existing `.dar` and `.dalf` files and access the various [`element`](daml_lf::element) they contain.  The
//! [`DamlElementVisitor`](daml_lf::element::DamlElementVisitor) provides a means to traverse these elements.
//!
//! The [`DamlPackages`](daml_util::package::DamlPackages) type provides the ability to extract Daml LF packages from an
//! existing ledger as a collection of [`DamlLfArchive`](daml_lf::DamlLfArchive) or combined into a single
//! [`DarFile`](daml_lf::DarFile) file.
//!
//! A [`DarFile`](daml_lf::DarFile) can be [applied](::daml_lf::DarFile::apply) to a function and also converted
//! [converted](daml_lf::DarFile::to_owned_archive) to be owned (bounded by `'static`) such that it is suitable to be
//! passed to a thread to async executor.
//!
//! ## Code Generation
//!
//! Rust representations of Daml Archives, Packages, Modules, Data, Templates & Choices can be generated or derived
//! using facilities provided by the [`daml-codegen`](::daml_codegen) and [`daml-derive`](::daml_derive) crates.
//!
//! The [`DamlTemplate`](daml_derive::DamlTemplate), [`DamlChoices`](daml_derive::DamlChoices),
//! [`DamlData`](daml_derive::DamlData), [`DamlVariant`](daml_derive::DamlVariant) & [`DamlEnum`](daml_derive::DamlEnum)
//! attribute procedural macros allow for Rust types to be annotated such that these can be used with Daml the GRPC
//! ledger API.  The
//! [`attributes`](https://github.com/fujiapple852/rust-daml-bindings/tree/master/daml-derive/tests/attribute) tests of
//! the [`daml-derive`](::daml_derive) crate provides several examples.
//!
//! The [`daml_codegen`](macro@daml_derive::daml_codegen) procedural macro is provided to enable generating Rust modules
//! and types for a complete `.dar` file.  A sample application which uses the [`daml_codegen`] macro in a `build.rs`
//! file is available in the
//! [`codegen-demo` example](https://github.com/fujiapple852/rust-daml-bindings/tree/master/examples/codegen-demo).
//! See also the
//! [`codegen`](https://github.com/fujiapple852/rust-daml-bindings/tree/master/daml-derive/tests/codegen/all_tests)
//! tests in the [`daml-derive`](::daml_derive) crate for examples of using the
//! [`daml_codegen`](macro@daml_derive::daml_codegen) macro.
//!
//! ## Tools
//!
//! The following standalone tools make use of the `daml` library:
//!
//! | crate                          | description                                                       |
//! |--------------------------------|-------------------------------------------------------------------|
//! | [daml-codegen](::daml_codegen) | Generate Rust GRPC API bindings from Daml archives                |
//! | [daml-bridge]                  | Daml JSON <> GRPC Ledger bridge                                   |
//! | [daml-oas]                     | Generate OpenAPI and AsyncAPI specifications from Daml dar files  |
//! | [daml-darn]                    | Daml Archive cli tool                                             |
//!
//! # Examples
//!
//! Further examples are available in the [`examples`](https://github.com/fujiapple852/rust-daml-bindings/tree/master/examples) directory.
//!
//! [daml-bridge]: https://docs.rs/daml-bridge/0.2.0/daml_bridge
//! [daml-oas]: https://docs.rs/daml-oas/0.2.0/daml-oas
//! [daml-darn]: https://docs.rs/daml-darn/0.2.0/daml-darn

#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]
#![doc(html_root_url = "https://docs.rs/daml/0.2.0")]

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
