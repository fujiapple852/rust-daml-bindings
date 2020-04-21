use crate::error::DarnError;
use crate::package_common::get_all_packages;
use anyhow::Result;
use daml::lf::element::{
    DamlAbsoluteDataRef, DamlArchive, DamlElementVisitor, DamlNonLocalDataRef, DamlVisitableElement,
};
use daml::lf::{DamlLfArchive, DarEncryptionType, DarFile, DarManifest, DarManifestFormat, DarManifestVersion};
use daml::util::archive::ExtendedPackageInfo;
use std::collections::{HashMap, HashSet};
use std::iter::{once, FromIterator};

const DAML_STDLIB_PACKAGE_NAME: &str = "daml-stdlib";

pub async fn download(uri: &str, output_dir: &str, token_key_path: Option<&str>, main_package_name: &str) -> Result<()> {
    let all_packages = get_all_packages(uri, token_key_path).await?;
    let working_dar = make_working_dar(all_packages);
    let dependencies: HashSet<ExtendedPackageInfo> = working_dar.apply(|arc| {
        let stdlib = ExtendedPackageInfo::find_from_archive(arc, |p| p.name == DAML_STDLIB_PACKAGE_NAME)
            .ok_or_else(|| DarnError::unknown_package(DAML_STDLIB_PACKAGE_NAME))?;
        extract_package_dependencies(arc, stdlib)
    })??;
    let extended_package_info = ExtendedPackageInfo::extract_from_dar(&working_dar)?;
    let main_package_info = extended_package_info
        .into_iter()
        .find(|p| p.package_name == main_package_name)
        .ok_or_else(|| DarnError::unknown_package(main_package_name))?;

    let included_package: HashMap<&str, &ExtendedPackageInfo> =
        once((main_package_info.package_id.as_str(), &main_package_info))
            .chain(dependencies.iter().map(|p| (p.package_id.as_str(), p)))
            .collect();

    // 3: we have to filter and rename each DamlLfArchive with the name we have just extracted from the package
    let renamed: HashMap<&str, DamlLfArchive> = once(working_dar.main)
        .chain(working_dar.dependencies)
        .filter_map(|mut p| {
            included_package.get_key_value(p.hash.as_str()).map(|(&id, &info)| {
                if let Some(version) = &info.version {
                    p.name = format!("{}-{}", info.package_name, version)
                } else {
                    p.name = info.package_name.to_string();
                }
                (id, p)
            })
        })
        .collect();

    // 4: we can extract the "main" package by name and build the final Dar
    let final_dar = make_final_dar(renamed, &main_package_info.package_id);
    let final_dar_name = format!("{}/{}.dar", output_dir, final_dar.main.name);
    final_dar.write_to_file(final_dar_name)?;
    Ok(())
}

#[derive(Default)]
struct PackageDependencyVisitor {
    refs: HashSet<ExtendedPackageInfo>,
}

impl DamlElementVisitor for PackageDependencyVisitor {
    fn pre_visit_non_local_data_ref(&mut self, non_local_data_ref: &DamlNonLocalDataRef<'_>) {
        self.refs.insert(ExtendedPackageInfo {
            package_id: non_local_data_ref.target_package_id.to_owned(),
            package_name: non_local_data_ref.target_package_name.to_owned(),
            version: None,
            language_version: "".to_owned(),
        });
    }

    fn pre_visit_absolute_data_ref(&mut self, absolute_data_ref: &DamlAbsoluteDataRef<'_>) {
        self.refs.insert(ExtendedPackageInfo {
            package_id: absolute_data_ref.package_id.to_owned(),
            package_name: absolute_data_ref.package_name.to_owned(),
            version: None,
            language_version: "".to_owned(),
        });
    }
}

fn extract_package_dependencies(
    archive: &DamlArchive<'_>,
    package_info: ExtendedPackageInfo,
) -> Result<HashSet<ExtendedPackageInfo>> {
    let package = archive
        .packages
        .values()
        .find(|&p| p.package_id == package_info.package_id)
        .ok_or_else(|| DarnError::unknown_package(&package_info.package_id))?;
    let mut visitor = PackageDependencyVisitor {
        refs: HashSet::<ExtendedPackageInfo>::default(),
    };
    package.accept(&mut visitor);
    visitor.refs.into_iter().fold(Ok(HashSet::from_iter(vec![package_info].into_iter())), |all_refs, name| {
        match all_refs {
            Ok(mut r) => {
                r.extend(extract_package_dependencies(archive, name)?);
                Ok(r)
            },
            Err(e) => Err(e),
        }
    })
}

fn make_working_dar(mut all_packages: Vec<DamlLfArchive>) -> DarFile {
    let (first, rest) = all_packages.try_swap_remove(0).map(|i| (i, all_packages)).unwrap();
    let manifest = DarManifest::new_implied(first.name.clone(), rest.iter().map(|n| n.name.clone()).collect());
    DarFile::new(manifest, first, rest)
}

fn make_final_dar(mut all_packages: HashMap<&str, DamlLfArchive>, package_id: &str) -> DarFile {
    let main = all_packages.remove(package_id).unwrap();
    let dependencies: Vec<DamlLfArchive> = all_packages.into_iter().map(|(_, v)| v).collect();
    let root_dir = format!("{}-{}", main.name, main.hash);
    let main_location = format!("{}/{}-{}.dalf", root_dir, main.name, main.hash);
    let dependency_locations =
        dependencies.iter().map(|d| format!("{}/{}-{}.dalf", root_dir, d.name, d.hash)).collect();
    let manifest = DarManifest::new(
        DarManifestVersion::V1,
        "darn",
        main_location,
        dependency_locations,
        DarManifestFormat::DamlLf,
        DarEncryptionType::NotEncrypted,
    );
    DarFile::new(manifest, main, dependencies)
}

trait TrySwapRemove<T>: Sized {
    fn try_swap_remove(&mut self, index: usize) -> Option<T>;
}

impl<T> TrySwapRemove<T> for Vec<T> {
    fn try_swap_remove(&mut self, index: usize) -> Option<T> {
        if index < self.len() {
            Some(self.swap_remove(index))
        } else {
            None
        }
    }
}
