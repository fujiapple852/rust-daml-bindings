mod archive_converter;
mod archive_payload;
mod data_data_box_checker;
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
use crate::{DamlLfArchive, DamlLfArchivePayload, DamlLfError, DamlLfHashFunction, DamlLfResult, DarFile};
use std::convert::TryFrom;

/// Convert a [`DarFile`] to a [`DamlArchive`] and map function `f` over it.
pub fn apply_dar<R, F>(dar: &DarFile, mut f: F) -> DamlLfResult<R>
where
    F: FnMut(&DamlArchive<'_>) -> R,
{
    let archive_payload = DamlArchivePayload::try_from(dar).map_err(DamlLfError::DamlLfConvertError)?;
    let archive_wrapper = DamlArchiveWrapper::new(&archive_payload);
    let archive = DamlArchive::try_from(archive_wrapper).map_err(DamlLfError::DamlLfConvertError)?;
    Ok(f(&archive))
}

/// Convert a [`DamlLfArchive`] to a [`DamlPackage`] and map function `f` over it.
pub fn apply_dalf<R, F>(dalf: &DamlLfArchive, mut f: F) -> DamlLfResult<R>
where
    F: FnMut(&DamlPackage<'_>) -> R,
{
    let package_payload = DamlPackagePayload::try_from(dalf).map_err(DamlLfError::DamlLfConvertError)?;
    let archive_payload = DamlArchivePayload::from_single_package(package_payload);
    let archive_wrapper = DamlArchiveWrapper::new(&archive_payload);
    let archive = DamlArchive::try_from(archive_wrapper).map_err(DamlLfError::DamlLfConvertError)?;
    Ok(f(archive.packages().values().next().req()?))
}

/// Convert a [`DamlLfArchivePayload`] to a [`DamlPackage`] and map function `f` over it.
pub fn apply_payload<R, F>(payload: DamlLfArchivePayload, mut f: F) -> DamlLfResult<R>
where
    F: FnMut(&DamlPackage<'_>) -> R,
{
    let dalf = DamlLfArchive::new("unnamed", payload, DamlLfHashFunction::SHA256, "");
    let package_payload = DamlPackagePayload::try_from(&dalf).map_err(DamlLfError::DamlLfConvertError)?;
    let archive_payload = DamlArchivePayload::from_single_package(package_payload);
    let archive_wrapper = DamlArchiveWrapper::new(&archive_payload);
    let archive = DamlArchive::try_from(archive_wrapper).map_err(DamlLfError::DamlLfConvertError)?;
    Ok(f(archive.packages().values().next().req()?))
}
