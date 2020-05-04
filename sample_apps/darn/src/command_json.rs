use crate::error::DarnError;
use anyhow::Result;
use daml::lf::DarFile;

pub(crate) fn json(dar_path: &str, package_opt: Option<&str>, module_opt: Option<&str>) -> Result<()> {
    let dar = DarFile::from_file(dar_path)?;
    let serialized = dar.apply(|archive| match (package_opt, module_opt) {
        (Some(search_package), Some(search_module)) => {
            let package =
                archive.packages().get(search_package).ok_or_else(|| DarnError::unknown_package(search_package))?;
            let module_search_path = search_module.split('.').collect::<Vec<_>>();
            let module = package
                .root_module()
                .child_module_path(&module_search_path)
                .ok_or_else(|| DarnError::unknown_module(search_module, search_package))?;
            serde_json::to_string_pretty(module).map_err(|e| DarnError::other_error(e.to_string()))
        },
        (Some(search_package), None) => {
            let package =
                archive.packages().get(search_package).ok_or_else(|| DarnError::unknown_package(search_package))?;
            serde_json::to_string_pretty(package).map_err(|e| DarnError::other_error(e.to_string()))
        },
        (None, _) => serde_json::to_string_pretty(archive).map_err(|e| DarnError::other_error(e.to_string())),
    })??;
    println!("{}", serialized);
    Ok(())
}
