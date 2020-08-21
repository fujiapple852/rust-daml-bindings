// find the main package by name
// let main_package_info = extended_package_info
//     .iter()
//     .find(|p| p.package_name == main_package_name)
//     .ok_or_else(|| DarnError::unknown_package(main_package_name))?;

// determine the set of dependencies on stdlib (we assume we depend on stdlib)
// let dependencies: HashSet<ExtendedPackageInfo> = working_dar.apply(|arc| {
//     let stdlib = ExtendedPackageInfo::find_from_archive(arc, |p| p.name == DAML_STDLIB_PACKAGE_NAME)
//         .ok_or_else(|| DarnError::unknown_package(DAML_STDLIB_PACKAGE_NAME))?;
//     extract_package_dependencies(arc, stdlib)
// })??;
// let included_package_infos: HashMap<&str, &ExtendedPackageInfo> =
//     once(&main_package_info).chain(dependencies.iter()).map(|p| (p.package_id.as_str(), p)).collect();
//
// // filter for only packages we wish to include and rename
// let included_packages: HashMap<&str, DamlLfArchive> = once(working_dar.main)
//     .chain(working_dar.dependencies)
//     .filter_map(|p| {
//         included_package_infos.get_key_value(p.hash()).map(|(&id, &info)| (id, rename_package(p, info)))
//     })
//     .collect();

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



// fn rename_package(mut package: DamlLfArchive, info: &ExtendedPackageInfo) -> DamlLfArchive {
//     if let Some(version) = &info.version {
//         package.name = format!("{}-{}", info.package_name, version)
//     } else {
//         package.name = info.package_name.to_string();
//     }
//     package
// }


