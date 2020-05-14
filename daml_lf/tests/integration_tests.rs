#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use daml_lf::element::DamlVisitableElement;
use daml_lf::element::{DamlElementVisitor, DamlEnum};
use daml_lf::DamlLfResult;
use daml_lf::DarFile;
use daml_lf::LanguageVersion;
use daml_lf::{DamlLfArchive, DamlLfHashFunction};
use daml_lf::{DarEncryptionType, DarManifestFormat, DarManifestVersion};
use std::collections::HashSet;

#[test]
fn test_dalf() -> DamlLfResult<()> {
    let archive =
        DamlLfArchive::from_file("../resources/testing_types_sandbox/archive/PingPongExample/PingPongExample.dalf")?;
    assert_eq!(&DamlLfHashFunction::SHA256, archive.hash_function());
    assert_eq!("2efa7ef832162fcb17abe86cd8675e31b8e641f25aba36a05098f7e9f4023d7e", archive.hash());
    assert_eq!("PingPongExample", archive.name());
    assert_eq!(LanguageVersion::V1_0, *archive.payload().language_version());
    assert!(archive.payload().contains_module("PingPong"));
    Ok(())
}

#[test]
pub fn test_legacy_dar() -> DamlLfResult<()> {
    let dar = DarFile::from_file("../resources/testing_types_sandbox/archive/PingPongExample.dar")?;
    assert_eq!(DarManifestVersion::Unknown, dar.manifest().version());
    assert_eq!("implied", dar.manifest().created_by());
    assert_eq!("PingPongExample/PingPongExample.dalf", dar.manifest().dalf_main());
    assert_eq!(&Vec::<String>::new(), dar.manifest().dalf_dependencies());
    assert_eq!(DarManifestFormat::Unknown, dar.manifest().format());
    assert_eq!(DarEncryptionType::Unknown, dar.manifest().encryption());
    assert_eq!(0, dar.dependencies().len());
    assert_eq!(LanguageVersion::V1_0, *dar.main().payload().language_version());
    assert!(dar.main().payload().contains_module("PingPong"));
    Ok(())
}

#[test]
pub fn test_fat_dar() -> DamlLfResult<()> {
    let dar = DarFile::from_file("test_resources/PingPongExampleFat.dar")?;
    assert_eq!(DarManifestVersion::V1, dar.manifest().version());
    assert_eq!("Digital Asset packager (DAML-GHC)", dar.manifest().created_by());
    assert_eq!("PingPongExample.dalf", dar.manifest().dalf_main());
    assert_eq!(&vec!["daml-prim.dalf", "daml-stdlib.dalf"], dar.manifest().dalf_dependencies());
    assert_eq!(DarManifestFormat::DamlLf, dar.manifest().format());
    assert_eq!(DarEncryptionType::NotEncrypted, dar.manifest().encryption());
    assert_eq!(2, dar.dependencies().len());
    assert_eq!(LanguageVersion::V1_3, *dar.main().payload().language_version());
    assert!(dar.main().payload().contains_module("PingPong"));
    Ok(())
}

#[test]
pub fn test_daml_lf_1_6() -> DamlLfResult<()> {
    let dar = DarFile::from_file("test_resources/TestingTypes-1_0_0-sdk_0_13_36-lf_1_6.dar")?;
    assert_eq!(LanguageVersion::V1_6, *dar.main().payload().language_version());
    Ok(())
}

#[test]
pub fn test_daml_lf_1_7() -> DamlLfResult<()> {
    let dar = DarFile::from_file("test_resources/TestingTypes-1_0_0-sdk_0_13_37-lf_1_7.dar")?;
    assert_eq!(LanguageVersion::V1_7, *dar.main().payload().language_version());
    Ok(())
}

#[test]
pub fn test_daml_lf_1_8() -> DamlLfResult<()> {
    let dar = DarFile::from_file("test_resources/TestingTypes-1_0_0-sdk_0_13_55-lf_1_8.dar")?;
    assert_eq!(LanguageVersion::V1_8, *dar.main().payload().language_version());
    Ok(())
}

#[test]
pub fn test_daml_sdk_1_0_x() -> DamlLfResult<()> {
    let dar = DarFile::from_file("test_resources/TestingTypes-1_0_0-sdk_1_0_1-lf_1_8.dar")?;
    assert_eq!(LanguageVersion::V1_8, *dar.main().payload().language_version());
    Ok(())
}

#[test]
pub fn test_daml_sdk_1_1_x() -> DamlLfResult<()> {
    let dar = DarFile::from_file("test_resources/TestingTypes-1_0_0-sdk_1_1_1-lf_1_8.dar")?;
    assert_eq!(LanguageVersion::V1_8, *dar.main().payload().language_version());
    Ok(())
}

#[test]
fn test_apply_dar() -> DamlLfResult<()> {
    let dar = DarFile::from_file("test_resources/TestingTypes-1_0_0-sdk_0_13_55-lf_1_8.dar")?;
    let name = dar.apply(|archive| archive.name().to_owned())?;
    assert_eq!("TestingTypes-1.0.0", name);
    Ok(())
}

#[test]
fn test_apply_dalf() -> DamlLfResult<()> {
    let dar = DarFile::from_file("test_resources/TestingTypes-1_0_0-sdk_0_13_55-lf_1_8.dar")?;
    let (name, version) = dar
        .dependencies
        .first()
        .unwrap()
        .apply(|package| (package.name().to_owned(), package.version().map(ToOwned::to_owned).unwrap()))?;
    assert_eq!("daml-prim", name);
    assert_eq!("0.0.0", version);
    Ok(())
}

#[test]
fn test_apply_payload() -> DamlLfResult<()> {
    let mut dar = DarFile::from_file("test_resources/TestingTypes-1_0_0-sdk_0_13_55-lf_1_8.dar")?;
    let payload = dar.dependencies.swap_remove(0).payload;
    let (name, version) =
        payload.apply(|package| (package.name().to_owned(), package.version().map(ToOwned::to_owned)))?;
    assert_eq!("daml-prim", name);
    assert_eq!(Some("0.0.0".to_owned()), version);
    Ok(())
}

#[test]
fn test_visitor() -> DamlLfResult<()> {
    #[derive(Default)]
    pub struct GatherEnumsVisitor(HashSet<String>);
    impl DamlElementVisitor for GatherEnumsVisitor {
        fn pre_visit_enum<'a>(&mut self, data_enum: &'a DamlEnum<'a>) {
            self.0.insert(data_enum.name().to_owned());
        }
    }
    let mut visitor = GatherEnumsVisitor::default();
    let dar = DarFile::from_file("test_resources/TestingTypes-1_0_0-sdk_0_13_55-lf_1_8.dar")?;
    dar.apply(|archive| archive.accept(&mut visitor))?;
    assert_eq!(true, visitor.0.contains("SimpleColor"));
    Ok(())
}
