use crate::error::DarnError;
use crate::package_common::get_all_packages;
use anyhow::Result;
use daml::api::data::package::DamlPackage;
use daml::lf::DamlLfArchivePayload;
use daml::lf::{
    DamlLfArchive, DamlLfHashFunction, DarEncryptionType, DarFile, DarManifest, DarManifestFormat, DarManifestVersion,
};
use daml::util::archive::ExtendedPackageInfo;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::CompressionMethod;

const UNKNOWN_LF_ARCHIVE_PREFIX: &str = "dep";
const MANIFEST_FILE_PATH: &str = "META-INF/MANIFEST.MF";

pub async fn download(
    uri: &str,
    output_dir: &str,
    token_key_path: Option<&str>,
    main_package_name: &str,
) -> Result<()> {
    // fetch all known packages ids & payload byte blobs from the ledger
    let all_packages_raw = get_all_packages(uri, token_key_path).await?;

    // convert the payload blobs into DamlLfArchives
    let all_packages = make_temp_packages(&all_packages_raw)?;

    // build a temporary working DarFile
    let working_dar = make_working_dar(all_packages);

    // build extended package info for all packages
    let extended_package_list = ExtendedPackageInfo::extract_from_dar(&working_dar)?;
    let extended_package_info: HashMap<&str, &ExtendedPackageInfo> =
        extended_package_list.iter().map(|i| (i.package_id.as_str(), i)).collect();

    // find the main package by name (TODO: or by id)
    let main_package_id = extended_package_info
        .values()
        .find_map(|p| {
            if p.package_name == main_package_name {
                Some(p.package_id.as_str())
            } else {
                None
            }
        })
        .ok_or_else(|| DarnError::unknown_package(main_package_name))?;

    // build a map of package id to payload byte blob
    let all_packages_bytes = make_final_packages(all_packages_raw)?;

    // write out a dar file
    write_dar(all_packages_bytes, main_package_id, &extended_package_info, output_dir)?;
    Ok(())
}

fn write_dar(
    all_packages: HashMap<String, Vec<u8>>,
    main_package_id: &str,
    package_info: &HashMap<&str, &ExtendedPackageInfo>,
    output_dir: &str,
) -> Result<()> {
    let main_package_info = package_info[main_package_id];
    let root_dir = format!("{}-{}", main_package_info.package_name, main_package_info.package_id);
    let main_location = make_dalf_path(&root_dir, main_package_info);
    let dependency_info: Vec<&ExtendedPackageInfo> = package_info
        .iter()
        .filter_map(|(&k, &v)| {
            if k == main_package_id {
                None
            } else {
                Some(v)
            }
        })
        .collect();
    let dependency_locations = dependency_info.iter().map(|info| make_dalf_path(&root_dir, info)).collect();
    let manifest = DarManifest::new(
        DarManifestVersion::V1,
        "darn",
        main_location,
        dependency_locations,
        DarManifestFormat::DamlLf,
        DarEncryptionType::NotEncrypted,
    );
    let final_dar_name: PathBuf = PathBuf::from(format!("{}/{}.dar", output_dir, main_package_info.package_name));
    final_dar_name.parent().map(std::fs::create_dir_all).transpose()?;
    let dar_file = std::fs::File::create(final_dar_name)?;
    let mut zip_writer = zip::ZipWriter::new(dar_file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
    zip_writer.set_comment("");
    let manifest_rendered = manifest.render();
    zip_writer.start_file(MANIFEST_FILE_PATH, options)?;
    zip_writer.write_all(manifest_rendered.as_bytes())?;
    for (package_id, dalf_bytes) in all_packages {
        let location = make_dalf_path(&root_dir, package_info[package_id.as_str()]);
        zip_writer.start_file(&location, options)?;
        zip_writer.write_all(dalf_bytes.as_slice())?;
    }
    zip_writer.finish()?;
    Ok(())
}

fn make_dalf_path(root_dir: &str, info: &ExtendedPackageInfo) -> String {
    format!("{}/{}-{}.dalf", root_dir, info.package_name, info.package_id)
}

// TODO lots of cloning here
fn make_final_packages(packages: Vec<DamlPackage>) -> Result<HashMap<String, Vec<u8>>> {
    packages
        .into_iter()
        .map(|d| {
            Ok((
                d.hash().to_owned(),
                DamlLfArchive::serialize_with_payload(
                    d.payload().to_owned(),
                    d.hash().to_owned(),
                    DamlLfHashFunction::SHA256,
                )?,
            ))
        })
        .collect::<Result<HashMap<_, _>>>()
}

// TODO lots of cloning here
fn make_temp_packages(packages: &[DamlPackage]) -> Result<Vec<DamlLfArchive>> {
    packages
        .iter()
        .map(|p| {
            let archive = DamlLfArchivePayload::from_bytes(p.payload().to_owned())?;
            let main = DamlLfArchive::new(
                format!("{}-{}", UNKNOWN_LF_ARCHIVE_PREFIX, &p.hash()),
                archive,
                DamlLfHashFunction::SHA256,
                p.hash().to_owned(),
            );
            Ok(main)
        })
        .collect()
}

fn make_working_dar(mut all_packages: Vec<DamlLfArchive>) -> DarFile {
    let (first, rest) = all_packages.try_swap_remove(0).map(|i| (i, all_packages)).unwrap();
    let manifest = DarManifest::new_implied(first.name.clone(), rest.iter().map(|n| n.name.clone()).collect());
    DarFile::new(manifest, first, rest)
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
