use core::iter;
use std::collections::HashMap;

use crate::convert::package_payload::{DamlPackagePayload, DamlPackageWrapper};
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::DarFile;
use std::convert::TryFrom;

///
#[derive(Debug, Clone, Copy)]
pub struct DamlArchiveWrapper<'a> {
    pub archive: &'a DamlArchivePayload<'a>,
}

impl<'a> DamlArchiveWrapper<'a> {
    pub const fn new(archive: &'a DamlArchivePayload<'_>) -> Self {
        Self {
            archive,
        }
    }

    pub const fn with_package(self, package: &'a DamlPackagePayload<'_>) -> DamlPackageWrapper<'a> {
        DamlPackageWrapper {
            archive: self.archive,
            package,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DamlArchivePayload<'a> {
    pub name: &'a str,
    pub packages: HashMap<&'a str, DamlPackagePayload<'a>>,
}

impl<'a> DamlArchivePayload<'a> {
    pub fn from_single_package(package: DamlPackagePayload<'a>) -> Self {
        Self {
            name: "",
            packages: vec![("", package)].into_iter().collect(),
        }
    }

    pub fn package_by_id(&self, name: &str) -> Option<&DamlPackagePayload<'_>> {
        self.packages.get(name)
    }
}

impl<'a> TryFrom<&'a DarFile> for DamlArchivePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(dar_file: &'a DarFile) -> DamlLfConvertResult<Self> {
        let name = &dar_file.main.name;
        let packages: HashMap<_, _> = dar_file
            .dependencies
            .iter()
            .chain(iter::once(&dar_file.main))
            .map(|p| Ok((p.hash.as_str(), DamlPackagePayload::try_from(p)?)))
            .collect::<DamlLfConvertResult<_>>()?;
        Ok(Self {
            name,
            packages,
        })
    }
}
