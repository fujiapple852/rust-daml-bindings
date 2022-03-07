//! A library for working with `Daml-LF`.
//!
//! Compiled `Daml` packages are represented as [`Daml-LF`](https://github.com/digital-asset/daml/tree/main/daml-lf) (
//! aka "Ledger Fragment") archives.  An archive is a protobuf serialized bytes array which is typically stored in a
//! `dalf` file.  Multiple `dalf` archives can be combined along with a manifest file into a `Dar` ("Daml Archive")
//! file.
//!
//! # Elements
//!
//! The [`element`] module contains a higher level representation of all Daml-LF types and provide several convenience
//! methods to simplify working with Daml-LF types over the raw types generated from the protobuf definitions.
//!
//! These types are typically constructed by converting an existing [`DarFile`], [`DamlLfArchive`] or
//! [`DamlLfArchivePayload`] which can be loaded from a file or downloaded from a Daml ledger.
//!
//! In the following example the `example.dar` is loaded from a file, converted to a `DamlArchive` and finally the id
//! of the main package is extracted:
//!
//! ```no_run
//! # use daml_lf::DarFile;
//! # use daml_lf::DamlLfResult;
//! # fn main() -> DamlLfResult<()> {
//! let dar = DarFile::from_file("Example.dar")?;
//! let package_id = dar.apply(|archive| archive.main_package_id().to_owned())?;
//! # Ok(())
//! # }
//! ```
//!
//! # Interning
//!
//! Interned data is automatically managed by the [`element`] items.  During conversion all [`element`] items borrow
//! interned data from the underlying source and so no additional allocations are required per element.
//!
//! However if an owned (i.e. bounded by `'static`) version is required, such as to pass to a thread or an async
//! executors, the method [`DarFile::to_owned_archive`] is provided to perform this conversion and allocate separate
//! copies of all interned data per [`element`] as required.
//!
//! The following example loads `Example.dar` from a file, converts it to an owned
//! [`DamlArchive`](`element::DamlArchive`) that is suitable to be passed to a new thread:
//!
//! ```no_run
//! # use daml_lf::DarFile;
//! # use daml_lf::DamlLfResult;
//! # use std::thread;
//! # fn main() -> DamlLfResult<()> {
//! let archive = DarFile::from_file("Example.dar")?.to_owned_archive()?;
//! thread::spawn(move || {
//!     dbg!(archive.name());
//! })
//! .join()
//! .unwrap();
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! The following features are defined:
//!
//! - `default` Includes all `Daml-LF` types _except_ Daml expressions.
//! - `full` Includes all `Daml-LF` types.
//!
//! # Downloading Daml Packages
//!
//! Serialized `Daml-LF` archives may also be retrieved from an existing ledger via the `GetPackage` method of the GRPC
//! `package_service` (see [here](https://github.com/digital-asset/daml/blob/main/ledger-api/grpc-definitions/com/daml/ledger/api/v1/package_service.proto)).
//! The `daml-grpc` create provides an implementation of this service in the [`daml_package_service`] module.
//!
//! The [`daml-util`] crate provides the [`DamlPackages`] helper to simplify downloading of packages form a Daml
//! ledger and converting to a [`DarFile`] or collections of [`DamlLfArchive`] or [`DamlLfArchivePayload`].
//!
//! # Versions
//!
//! This library supports all Daml-LF [`LanguageVersion`] from [`LanguageVersion::V1_0`] up to
//! [`LanguageVersion::V1_14`].
//!
//! [`daml-util`]: https://docs.rs/daml-util/0.2.1/daml_util/
//! [`DamlPackages`]: https://docs.rs/daml-util/0.2.1/daml_util/package/struct.DamlPackages.html
//! [`daml_package_service`]: https://docs.rs/daml-grpc/0.2.1/daml_grpc/service/struct.DamlPackageService.html
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
#![doc(html_root_url = "https://docs.rs/daml-lf/0.2.1")]

mod archive;
mod convert;
mod dar;
mod error;
mod lf_protobuf;
mod manifest;
mod package_info;
mod payload;
mod version;

/// Representation of Daml types.
pub mod element;

// reexport types
pub use archive::{DamlLfArchive, DamlLfHashFunction, DEFAULT_ARCHIVE_NAME};
pub use dar::DarFile;
pub use error::{DamlLfError, DamlLfResult};
pub use manifest::{DarEncryptionType, DarManifest, DarManifestFormat, DarManifestVersion};
pub use package_info::PackageInfo;
pub use payload::{DamlLfArchivePayload, DamlLfPackage};
pub use version::{LanguageFeatureVersion, LanguageV1MinorVersion, LanguageVersion};
