use core::iter;
use std::collections::HashMap;

use crate::convert::archive::wrapper::payload::package_payload::DamlPackagePayload;
use daml_lf::DarFile;

#[derive(Debug, PartialEq)]
pub struct DamlArchivePayload<'a> {
    pub name: &'a str,
    pub packages: HashMap<&'a str, DamlPackagePayload<'a>>,
}

impl<'a> DamlArchivePayload<'a> {
    pub fn package_by_id(&self, name: &str) -> Option<&DamlPackagePayload> {
        self.packages.get(name)
    }
}

impl<'a> From<&'a DarFile> for DamlArchivePayload<'a> {
    fn from(dar_file: &'a DarFile) -> Self {
        let name = &dar_file.main.name;
        let packages: HashMap<_, _> = dar_file
            .dependencies
            .iter()
            .chain(iter::once(&dar_file.main))
            .map(|p| (p.hash.as_str(), DamlPackagePayload::from(p)))
            .collect();
        Self {
            name,
            packages,
        }
    }
}
