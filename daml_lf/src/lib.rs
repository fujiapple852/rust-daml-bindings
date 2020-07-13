//! API for working with `DAML-LF` packages.
//!
//! Compiled `DAML` packages are represented as `DAML-LF` ("Ledger Fragment") archives.  An archive is a protobuf
//! serialized bytes array which is typically stored in a `dalf` file.  Multiple `dalf` archives can be combined along
//! with a manifest file into a `Dar` ("DAML Archive") file.
//!
//! Serialized `DAML-LF` archives may also be retrieved from an existing ledger via the `GetPackage` method of the GRPC
//! `package_service` (see [here](https://github.com/digital-asset/daml/blob/master/ledger-api/grpc-definitions/com/digitalasset/ledger/api/v1/package_service.proto)).
//! The `daml_grpc` create provides an implementation of this service in the [`daml_package_service`] module.
//!
//! See [here](https://github.com/digital-asset/daml/tree/master/daml-lf) for full details of DAML-LF.
//!
//! [`daml_package_service`]: ../daml_grpc/service/struct.DamlPackageService.html
#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::module_name_repetitions,
    clippy::use_self,
    clippy::cast_sign_loss,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc
)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]

mod archive;
mod convert;
mod dar;
mod error;
mod lf_protobuf;
mod manifest;
mod owned;
mod payload;
mod version;

/// Representation of DAML types.
pub mod element;

// reexport types
pub use archive::{DamlLfArchive, DamlLfHashFunction, DEFAULT_ARCHIVE_NAME};
pub use dar::DarFile;
pub use error::{DamlLfError, DamlLfResult};
pub use manifest::{DarEncryptionType, DarManifest, DarManifestFormat, DarManifestVersion};
pub use payload::{DamlLfArchivePayload, DamlLfPackage};
pub use version::{LanguageFeatureVersion, LanguageV1MinorVersion, LanguageVersion};
