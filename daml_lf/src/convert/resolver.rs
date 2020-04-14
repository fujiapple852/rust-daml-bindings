use crate::convert::data_payload::{DamlDataEnrichedPayload, DamlDataWrapper};
use crate::convert::package_payload::DamlPackagePayload;
use crate::convert::type_payload::DamlDataRefWrapper;
use crate::convert::wrapper::DamlPayloadDataWrapper;
use crate::error::{DamlLfConvertError, DamlLfConvertResult};

/// Resolve a `DamlDataRefWrapper` to a `DamlDataWrapper`.
///
/// A `DamlDataRefPayload` "refers to" a `DamlDataPayload` which may live any `DamlModulePayload` of any
/// `DamlPackagePayload` in the current `DamlArchivePayload`.
///
/// This function attempts to find and return a `DamlDataWrapper` from the parent `DamlArchivePayload` which
/// matches the package, module & data name specified in the supplied `DamlDataRefWrapper` and returns a
/// `DamlCodeGenError` if no such entry exists.
pub fn resolve_data_ref<'a>(data_ref: DamlDataRefWrapper<'a>) -> DamlLfConvertResult<DamlDataWrapper<'_>> {
    let source_resolver = data_ref.context.package;
    let source_data_type_name = data_ref.payload.data_name.resolve(source_resolver)?;
    let target_package_id = data_ref.payload.package_ref.resolve(source_resolver)?;

    // Extract the target package from the parent archive
    let target_package: &DamlPackagePayload<'a> = data_ref
        .context
        .archive
        .package_by_id(target_package_id)
        .ok_or_else(|| DamlLfConvertError::UnknownPackage(target_package_id.to_owned()))?;

    // Extract the target module from the target package
    let target_module = target_package
        .module_by_name(&data_ref.payload.module_path.resolve(source_resolver)?.join("."))
        .ok_or_else(|| DamlLfConvertError::UnknownModule(data_ref.payload.module_path.to_string()))?;

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
    let target_data = DamlPayloadDataWrapper {
        archive: data_ref.context.archive,
        package: target_package,
        module: target_module,
        data: target_data_type,
    };
    Ok(DamlDataWrapper::with_data(target_data, DamlDataEnrichedPayload::from_data_wrapper(target_data)))
}
