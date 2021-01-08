use crate::convert::archive_payload::DamlArchivePayload;
use crate::convert::data_payload::{DamlDataEnrichedPayload, DamlDataWrapper};
use crate::convert::interned::{InternableDottedName, PackageInternedResolver};
use crate::convert::package_payload::DamlPackagePayload;
use crate::convert::type_payload::{DamlPackageRefPayload, DamlSynWrapper, DamlTyConWrapper};
use crate::convert::wrapper::{DamlPayloadParentContext, DamlPayloadParentContextType};
use crate::error::{DamlLfConvertError, DamlLfConvertResult};

/// Resolve a `DamlTyConWrapper` to a `DamlDataWrapper`.
///
/// A `DamlTyConPayload` "refers to" a `DamlDataPayload` which may live any `DamlModulePayload` of any
/// `DamlPackagePayload` in the current `DamlArchivePayload`.
///
/// This function attempts to find and return a `DamlDataWrapper` from the parent `DamlArchivePayload` which
/// matches the package, module & data name specified in the supplied `DamlTyConWrapper` and returns a
/// `DamlCodeGenError` if no such entry exists.
pub fn resolve_tycon(tycon: DamlTyConWrapper<'_>) -> DamlLfConvertResult<DamlDataWrapper<'_>> {
    resolve(
        tycon.context.package,
        tycon.context.archive,
        &tycon.payload.package_ref,
        tycon.payload.module_path,
        tycon.payload.data_name,
    )
}

/// Resolve a `DamlSynWrapper` to a `DamlDataWrapper`.
pub fn resolve_syn(tycon: DamlSynWrapper<'_>) -> DamlLfConvertResult<DamlDataWrapper<'_>> {
    resolve(
        tycon.context.package,
        tycon.context.archive,
        &tycon.payload.tysyn.package_ref,
        tycon.payload.tysyn.module_path,
        tycon.payload.tysyn.data_name,
    )
}

fn resolve<'a, R: PackageInternedResolver>(
    intern_resolver: &R,
    archive: &'a DamlArchivePayload<'a>,
    package_ref: &'a DamlPackageRefPayload<'a>,
    module_path: InternableDottedName<'a>,
    data_name: InternableDottedName<'a>,
) -> DamlLfConvertResult<DamlDataWrapper<'a>> {
    let source_data_type_name = data_name.resolve(intern_resolver)?;
    let target_package_id = package_ref.resolve(intern_resolver)?;
    let target_module_path = module_path.resolve(intern_resolver)?.join(".");

    // Extract the target package from the parent archive
    let target_package: &DamlPackagePayload<'_> = archive
        .package_by_id(&target_package_id)
        .ok_or_else(|| DamlLfConvertError::UnknownPackage(target_package_id.to_string()))?;

    // Extract the target module from the target package
    let target_module = target_package
        .module_by_name(&target_module_path)
        .ok_or(DamlLfConvertError::UnknownModule(target_module_path))?;

    // Find the target data from the target module
    let data_types_iter =
        target_module.data_types.iter().map(|dt| dt.name().resolve(target_package).map(|name| (name, dt)));
    let target_data_type = itertools::process_results(data_types_iter, |mut iter| {
        iter.find_map(|(name, dt)| {
            if name == source_data_type_name {
                Some(dt)
            } else {
                None
            }
        })
    })?
    .ok_or_else(|| DamlLfConvertError::UnknownData(source_data_type_name.join(".")))?;

    // Return the target data wrapped in the target package and module context
    let target_data = DamlPayloadParentContext {
        archive,
        package: target_package,
        module: target_module,
        parent: DamlPayloadParentContextType::Data(target_data_type),
    };
    Ok(DamlDataWrapper::with_data(target_data, DamlDataEnrichedPayload::from_data_wrapper(target_data)?))
}
