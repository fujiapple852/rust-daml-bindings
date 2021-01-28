use crate::element::{DamlArchive, DamlPackage};
use crate::{DamlLfResult, DarFile, LanguageVersion};

/// Information about a `DamlPackage`.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct PackageInfo {
    pub package_id: String,
    pub package_name: String,
    pub version: Option<String>,
    pub language_version: LanguageVersion,
}

impl PackageInfo {
    pub fn new(
        package_id: impl Into<String>,
        package_name: impl Into<String>,
        version: impl Into<Option<String>>,
        language_version: LanguageVersion,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            package_name: package_name.into(),
            version: version.into(),
            language_version: language_version.into(),
        }
    }

    /// Extract extended information about the packages in a `DarFile`.
    ///
    /// This is a relatively expensive operation as it involves creating a full `DamlArchive` representation of the
    /// given `DarFile`.
    ///
    /// This is useful when examining a `DarFile` which has been constructed from a collection of packages (i.e. dalf
    /// files or packages downloaded from a ledger) which do not have metadata.
    pub fn extract_from_dar(dar: &DarFile) -> DamlLfResult<Vec<Self>> {
        dar.apply(Self::extract_from_archive)
    }

    /// Extract extended information about the packages in a `DamlArchive`.
    pub fn extract_from_archive(archive: &DamlArchive<'_>) -> Vec<Self> {
        archive.packages().map(Self::extract_from_package).collect()
    }

    /// Extract extended information from a `DamlPackage`.
    pub fn extract_from_package(package: &DamlPackage<'_>) -> Self {
        Self::new(
            package.package_id(),
            package.name(),
            package.version().map(ToOwned::to_owned),
            package.language_version(),
        )
    }

    /// Extract extended package information about the package which matches the predicate.
    pub fn find_from_archive<F>(archive: &DamlArchive<'_>, f: F) -> Option<Self>
    where
        F: Fn(&DamlPackage<'_>) -> bool,
    {
        archive.packages().find_map(|p| {
            if f(p) {
                Some(Self::extract_from_package(p))
            } else {
                None
            }
        })
    }
}
