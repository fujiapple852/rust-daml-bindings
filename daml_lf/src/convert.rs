mod archive_converter;
mod archive_payload;
mod data_box_checker;
mod data_payload;
mod field_payload;
mod interned;
mod module_payload;
mod package_payload;
mod resolver;
mod type_payload;
mod typevar_payload;
mod util;
mod wrapper;

#[cfg(feature = "full")]
mod defvalue_payload;
#[cfg(feature = "full")]
mod expr_converter;
#[cfg(feature = "full")]
mod expr_payload;

use crate::convert::archive_payload::{DamlArchivePayload, DamlArchiveWrapper};
use crate::convert::package_payload::DamlPackagePayload;
use crate::convert::util::Required;
use crate::element::{DamlArchive, DamlPackage};
use crate::owned::ToStatic;
use crate::{DamlLfArchive, DamlLfArchivePayload, DamlLfHashFunction, DamlLfResult, DarFile};
use std::convert::TryFrom;

/// Create an owned [`DamlArchive`] from a [`DarFile`].
pub fn to_owned_archive(dar: &DarFile) -> DamlLfResult<DamlArchive<'static>> {
    let archive_payload = DamlArchivePayload::try_from(dar)?;
    let archive = DamlArchive::try_from(DamlArchiveWrapper::new(&archive_payload))?;
    Ok(archive.to_static())
}

/// Convert a [`DarFile`] to a [`DamlArchive`] and map function `f` over it.
pub fn apply_dar<R, F>(dar: &DarFile, f: F) -> DamlLfResult<R>
where
    F: FnOnce(&DamlArchive<'_>) -> R,
{
    let archive_payload = DamlArchivePayload::try_from(dar)?;
    let archive_wrapper = DamlArchiveWrapper::new(&archive_payload);
    let archive = DamlArchive::try_from(archive_wrapper)?;
    Ok(f(&archive))
}

/// Create a [`DamlArchive`] from a [`DamlLfArchive`] and apply it to `f`.
pub fn apply_dalf<R, F>(dalf: &DamlLfArchive, f: F) -> DamlLfResult<R>
where
    F: FnOnce(&DamlPackage<'_>) -> R,
{
    let package_payload = DamlPackagePayload::try_from(dalf)?;
    let archive_payload = DamlArchivePayload::from_single_package(package_payload);
    let archive_wrapper = DamlArchiveWrapper::new(&archive_payload);
    let archive = DamlArchive::try_from(archive_wrapper)?;
    let package = archive.packages().next().req()?;
    Ok(f(package))
}

/// Create a [`DamlArchive`] from a [`DamlLfArchivePayload`] and apply it to `f`.
pub fn apply_payload<R, F>(payload: DamlLfArchivePayload, f: F) -> DamlLfResult<R>
where
    F: FnOnce(&DamlPackage<'_>) -> R,
{
    let dalf = DamlLfArchive::new("unnamed", payload, DamlLfHashFunction::Sha256, "");
    let package_payload = DamlPackagePayload::try_from(&dalf)?;
    let archive_payload = DamlArchivePayload::from_single_package(package_payload);
    let archive_wrapper = DamlArchiveWrapper::new(&archive_payload);
    let archive = DamlArchive::try_from(archive_wrapper)?;
    let package = archive.packages().next().req()?;
    Ok(f(package))
}
