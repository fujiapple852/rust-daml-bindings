//! API for working with `DAML-LF` packages.
//!
//! Compiled `DAML` packages are represented as `DAML-LF` ("Ledger Fragment") archives.  An archive is a protobuf
//! serialized bytes array which is typically stored in a `dalf` file.  Multiple `dalf` archives can be combined along
//! with a manifest file into a `Dar` ("DAML Archive") file.
//!
//! Serialized `DAML-LF` archives may also be retrieved from an existing ledger via the `GetPackage` method of the GRPC
//! `package_service` (see [here](https://github.com/digital-asset/daml/blob/master/ledger-api/grpc-definitions/com/digitalasset/ledger/api/v1/package_service.proto)).
//! The `daml_ledger_api` create provides an implementation of this service in the [`daml_package_service`] module.
//!
//! See [here](https://github.com/digital-asset/daml/tree/master/daml-lf) for full details of DAML-LF.
//!
//! [`daml_package_service`]: ../daml_ledger_api/service/struct.DamlPackageService.html

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::use_self, clippy::cast_sign_loss, clippy::must_use_candidate)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]

mod archive;
mod dar;
mod error;
mod manifest;
mod payload;
pub mod protobuf_autogen;
mod version;

// reexport all types (flatten module name space)
pub use archive::*;
pub use dar::*;
pub use error::*;
pub use manifest::*;
pub use payload::*;
pub use version::*;
