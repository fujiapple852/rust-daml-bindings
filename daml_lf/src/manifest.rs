use crate::error::{DamlLfError, DamlLfResult};
use std::convert::Into;
use yaml_rust::YamlLoader;

const MANIFEST_VERSION_KEY: &str = "Manifest-Version";
const CREATED_BY_KEY: &str = "Created-By";
const DALF_MAIN_KEY: &str = "Main-Dalf";
const DALFS_KEY: &str = "Dalfs";
const FORMAT_KEY: &str = "Format";
const ENCRYPTION_KEY: &str = "Encryption";
const VERSION_1_VALUE: &str = "1.0";
const NON_ENCRYPTED_VALUE: &str = "non-encrypted";
const DAML_LF_VALUE: &str = "daml-lf";

/// The version of a dar file manifest.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DarManifestVersion {
    Unknown,
    V1,
}

/// The format of the archives in a dar file.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DarManifestFormat {
    Unknown,
    DamlLf,
}

/// The encryption type of the archives in a dar file.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DarEncryptionType {
    Unknown,
    NotEncrypted,
}

/// Represents a manifest file found inside `dar` files.
///
/// A `dar` `manifest` file contains the following fields:
///
/// - `Manifest-Version`: the version of the manifest file (optional, defaults to [`Unknown`])
/// - `Created-By`: describes what created the `dar` file containing this manifest file (optional, , default to empty
///   string)
/// - `Main-Dalf`: the name of the `main` `dalf` file within the `dar` file (mandatory)
/// - `Dalfs`: a comma separated list of `dalf` files within this `dar` file (mandatory)
/// - `Format`: the format of the `dalf` files in this `dar` archive (mandatory)
/// - `Encryption`: the encryption type of the `dalf` files in this `dar` archive (mandatory)
///
/// Note that the `main` `dalf` file MUST also be provided in the `Dalfs` attribute and so that attribute will never
/// be empty.
///
/// [`Unknown`]: DarManifestVersion::Unknown
#[derive(Debug, Clone)]
pub struct DarManifest {
    version: DarManifestVersion,
    created_by: String,
    dalf_main: String,
    dalf_dependencies: Vec<String>,
    format: DarManifestFormat,
    encryption: DarEncryptionType,
}

impl DarManifest {
    /// Crate a `DarManifest`.
    pub fn new(
        version: impl Into<DarManifestVersion>,
        created_by: impl Into<String>,
        dalf_main: impl Into<String>,
        dalf_dependencies: Vec<String>,
        format: impl Into<DarManifestFormat>,
        encryption: impl Into<DarEncryptionType>,
    ) -> Self {
        Self {
            version: version.into(),
            created_by: created_by.into(),
            dalf_main: dalf_main.into(),
            dalf_dependencies,
            format: format.into(),
            encryption: encryption.into(),
        }
    }

    /// Create a `DarManifest` from the supplied `main` and `dalf_dependencies` `dalf` files.
    pub fn new_implied(dalf_main: impl Into<String>, dalf_dependencies: Vec<String>) -> Self {
        Self::new(
            DarManifestVersion::Unknown,
            "implied",
            dalf_main,
            dalf_dependencies,
            DarManifestFormat::Unknown,
            DarEncryptionType::Unknown,
        )
    }

    /// Create a `DarManifest` from the supplied `manifest` string.
    ///
    /// Note that all `dalf` names are stripped of all whitespace.
    ///
    /// # Errors
    ///
    /// If the provided `manifest` string cannot be parsed into newline `key: value` pairs then [`IOError`] will be
    /// returned.
    ///
    /// If the parsed `manifest` has an invalid format (such as missing a mandatory key) then [`DarParseError`] will
    /// be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use daml_lf::{DarManifestVersion, DarManifestFormat, DarEncryptionType, DarManifest};
    /// # use daml_lf::DamlLfResult;
    /// # fn main() -> DamlLfResult<()> {
    /// let manifest_str = "
    ///            Main-Dalf: A.dalf
    ///            Dalfs: A.dalf
    ///            Format: daml-lf
    ///            Encryption: non-encrypted";
    /// let manifest = DarManifest::parse(&manifest_str[..])?;
    /// assert_eq!(DarManifestVersion::Unknown, manifest.version());
    /// assert_eq!("", manifest.created_by());
    /// assert_eq!("A.dalf", manifest.dalf_main());
    /// assert_eq!(&Vec::<String>::new(), manifest.dalf_dependencies());
    /// assert_eq!(DarManifestFormat::DamlLf, manifest.format());
    /// assert_eq!(DarEncryptionType::NotEncrypted, manifest.encryption());
    /// # Ok(())
    /// # }
    /// ```
    /// [`IOError`]: DamlLfError::IOError
    /// [`DarParseError`]: DamlLfError::DarParseError
    pub fn parse(manifest: &str) -> DamlLfResult<Self> {
        let docs = YamlLoader::load_from_str(manifest)?;
        let doc = docs.first().ok_or_else(|| DamlLfError::new_dar_parse_error("unexpected manifest format"))?;

        let manifest_version = match doc[MANIFEST_VERSION_KEY].as_f64() {
            Some(s) if format!("{:.*}", 1, s) == VERSION_1_VALUE => Ok(DarManifestVersion::V1),
            Some(s) => Err(DamlLfError::new_dar_parse_error(format!(
                "unexpected value for {}, found {}",
                MANIFEST_VERSION_KEY, s
            ))),
            None => Ok(DarManifestVersion::Unknown),
        }?;

        let created_by = doc[CREATED_BY_KEY].as_str().map_or_else(|| "", |s| s);

        let dalf_main = doc[DALF_MAIN_KEY]
            .as_str()
            .map(strip_string)
            .ok_or_else(|| DamlLfError::new_dar_parse_error(format!("key {} not found", DALF_MAIN_KEY)))?;

        let dalf_dependencies = match doc[DALFS_KEY].as_str() {
            Some(s) => Ok(s
                .split(',')
                .filter_map(|dalf: &str| {
                    let stripped_dalf = strip_string(dalf);
                    if stripped_dalf == dalf_main {
                        None
                    } else {
                        Some(stripped_dalf)
                    }
                })
                .collect()),
            _ => Err(DamlLfError::new_dar_parse_error(format!("key {} not found", DALFS_KEY))),
        }?;

        let format = match doc[FORMAT_KEY].as_str() {
            Some(s) if s.to_lowercase() == DAML_LF_VALUE => Ok(DarManifestFormat::DamlLf),
            Some(s) =>
                Err(DamlLfError::new_dar_parse_error(format!("unexpected value for {}, found {}", DAML_LF_VALUE, s))),
            _ => Err(DamlLfError::new_dar_parse_error(format!("key {} not found", DAML_LF_VALUE))),
        }?;

        let encryption = match doc[ENCRYPTION_KEY].as_str() {
            Some(s) if s.to_lowercase() == NON_ENCRYPTED_VALUE => Ok(DarEncryptionType::NotEncrypted),
            Some(s) => Err(DamlLfError::new_dar_parse_error(format!(
                "unexpected value for {}, found {}",
                NON_ENCRYPTED_VALUE, s
            ))),
            _ => Err(DamlLfError::new_dar_parse_error(format!("key {} not found", NON_ENCRYPTED_VALUE))),
        }?;

        Ok(Self::new(manifest_version, created_by, dalf_main, dalf_dependencies, format, encryption))
    }

    /// The version of the manifest.
    pub fn version(&self) -> DarManifestVersion {
        self.version
    }

    /// Describes who created the `dar` file which contains this manifest file.
    pub fn created_by(&self) -> &str {
        &self.created_by
    }

    /// The name of the `main` `dalf` archive within the `dar` file containing this manifest file.
    pub fn dalf_main(&self) -> &str {
        &self.dalf_main
    }

    /// A list of names of the `dalf_dependencies` `dalf` archives within the `dar` file containing this manifest file.
    pub fn dalf_dependencies(&self) -> &Vec<String> {
        &self.dalf_dependencies
    }

    /// The format of the `dar` which contains this manifest file.
    pub fn format(&self) -> DarManifestFormat {
        self.format
    }

    /// The encryption type of the `dar` which contains this manifest file.
    pub fn encryption(&self) -> DarEncryptionType {
        self.encryption
    }
}

fn strip_string(s: impl AsRef<str>) -> String {
    s.as_ref().chars().filter(|&c| !char::is_whitespace(c)).collect()
}

#[cfg(test)]
mod test {
    use crate::error::{DamlLfError, DamlLfResult};
    use crate::manifest::*;
    use trim_margin::MarginTrimmable;

    #[test]
    pub fn test_split_dalfs() -> DamlLfResult<()> {
        let manifest_str = "
            |Manifest-Version: 1.0
            |Created-By: Digital Asset packager (DAML-GHC)
            |Main-Dalf: com.digitalasset.daml.lf.archive:DarReaderTest:0.1.dalf
            |Dalfs: com.digitalasset.daml.lf.archive:DarReaderTest:0.1.dalf, daml-pri
            | m.dalf
            |Format: daml-lf
            |Encryption: non-encrypted"
            .trim_margin()
            .expect("invalid test string");
        let manifest = DarManifest::parse(&manifest_str[..])?;
        assert_eq!(DarManifestVersion::V1, manifest.version());
        assert_eq!("Digital Asset packager (DAML-GHC)", manifest.created_by());
        assert_eq!("com.digitalasset.daml.lf.archive:DarReaderTest:0.1.dalf", manifest.dalf_main());
        assert_eq!(&vec!["daml-prim.dalf"], manifest.dalf_dependencies());
        assert_eq!(DarManifestFormat::DamlLf, manifest.format());
        assert_eq!(DarEncryptionType::NotEncrypted, manifest.encryption());
        Ok(())
    }

    #[test]
    pub fn test_split_all_dalf() -> DamlLfResult<()> {
        let manifest_str = "
            |Manifest-Version: 1.0
            |Created-By: Digital Asset packager (DAML-GHC)
            |Sdk-Version: 0.13.16
            |Main-Dalf: test-0.0.1-7390c3f7a0f5c4aed2cf8da2dc757885ac20ab8f2eb616a1ed
            | b7cf57f8161d3a/test.dalf
            |Dalfs: test-0.0.1-7390c3f7a0f5c4aed2cf8da2dc757885ac20ab8f2eb616a1edb7cf
            | 57f8161d3a/test.dalf, test-0.0.1-7390c3f7a0f5c4aed2cf8da2dc757885ac20ab
            | 8f2eb616a1edb7cf57f8161d3a/daml-prim.dalf, test-0.0.1-7390c3f7a0f5c4aed
            | 2cf8da2dc757885ac20ab8f2eb616a1edb7cf57f8161d3a/daml-stdlib.dalf
            |Format: daml-lf
            |Encryption: non-encrypted"
            .trim_margin()
            .expect("invalid test string");
        let manifest = DarManifest::parse(&manifest_str[..])?;
        assert_eq!(DarManifestVersion::V1, manifest.version());
        assert_eq!("Digital Asset packager (DAML-GHC)", manifest.created_by());
        assert_eq!(
            "test-0.0.1-7390c3f7a0f5c4aed2cf8da2dc757885ac20ab8f2eb616a1edb7cf57f8161d3a/test.dalf",
            manifest.dalf_main()
        );
        assert_eq!(
            &vec![
                "test-0.0.1-7390c3f7a0f5c4aed2cf8da2dc757885ac20ab8f2eb616a1edb7cf57f8161d3a/daml-prim.dalf",
                "test-0.0.1-7390c3f7a0f5c4aed2cf8da2dc757885ac20ab8f2eb616a1edb7cf57f8161d3a/daml-stdlib.dalf"
            ],
            manifest.dalf_dependencies()
        );
        assert_eq!(DarManifestFormat::DamlLf, manifest.format());
        assert_eq!(DarEncryptionType::NotEncrypted, manifest.encryption());
        Ok(())
    }

    #[test]
    pub fn test_multiple_dalfs() -> DamlLfResult<()> {
        let manifest_str = "
            |Main-Dalf: A.dalf
            |Dalfs: B.dalf, C.dalf, A.dalf, E.dalf
            |Format: daml-lf
            |Encryption: non-encrypted"
            .trim_margin()
            .expect("invalid test string");
        let manifest = DarManifest::parse(&manifest_str[..])?;
        assert_eq!(DarManifestVersion::Unknown, manifest.version());
        assert_eq!("", manifest.created_by());
        assert_eq!("A.dalf", manifest.dalf_main());
        assert_eq!(&vec!["B.dalf", "C.dalf", "E.dalf"], manifest.dalf_dependencies());
        assert_eq!(DarManifestFormat::DamlLf, manifest.format());
        assert_eq!(DarEncryptionType::NotEncrypted, manifest.encryption());
        Ok(())
    }

    #[test]
    pub fn test_single_main_dalf() -> DamlLfResult<()> {
        let manifest_str = "
            |Main-Dalf: A.dalf
            |Dalfs: A.dalf
            |Format: daml-lf
            |Encryption: non-encrypted"
            .trim_margin()
            .expect("invalid test string");
        let manifest = DarManifest::parse(&manifest_str[..])?;
        assert_eq!(DarManifestVersion::Unknown, manifest.version());
        assert_eq!("", manifest.created_by());
        assert_eq!("A.dalf", manifest.dalf_main());
        assert_eq!(&Vec::<String>::new(), manifest.dalf_dependencies());
        assert_eq!(DarManifestFormat::DamlLf, manifest.format());
        assert_eq!(DarEncryptionType::NotEncrypted, manifest.encryption());
        Ok(())
    }

    #[test]
    pub fn test_invalid_format() -> DamlLfResult<()> {
        let manifest_str = "
            |Main-Dalf: A.dalf
            |Dalfs: B.dalf, C.dalf, A.dalf, E.dalf
            |Format: anything-different-from-daml-lf
            |Encryption: non-encrypted"
            .trim_margin()
            .expect("invalid test string");
        let manifest = DarManifest::parse(&manifest_str[..]);
        match manifest.err().expect("expected failure") {
            DamlLfError::DarParseError(s) =>
                assert_eq!("unexpected value for daml-lf, found anything-different-from-daml-lf", s),
            _ => panic!("expected failure"),
        }
        Ok(())
    }
}
