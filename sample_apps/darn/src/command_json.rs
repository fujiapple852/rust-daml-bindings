use anyhow::Result;
use daml::lf::DarFile;

pub(crate) fn json(dar_path: &str, package_opt: Option<&str>, module_opt: Option<&str>) -> Result<()> {
    let dar = DarFile::from_file(dar_path)?;
    Ok(dar.apply(|archive| {
        let serialized = match (package_opt, module_opt) {
            (Some(search_package), Some(search_module)) => archive
                .packages
                .get(search_package)
                .and_then(|p| p.root_module.child_module(&search_module.split('.').collect::<Vec<_>>()))
                .and_then(|m| serde_json::to_string_pretty(m).ok()),
            (Some(search_package), None) =>
                archive.packages.get(search_package).and_then(|p| serde_json::to_string_pretty(p).ok()),
            (None, _) => serde_json::to_string_pretty(archive).ok(),
        }
        .expect("failed to serialize archive");
        println!("{}", serialized);
    })?)
}
