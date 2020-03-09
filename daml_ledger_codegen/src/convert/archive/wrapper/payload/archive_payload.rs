use core::iter;
use std::collections::HashMap;

use crate::convert::archive::wrapper::payload::package_payload::DamlPackagePayload;
use crate::convert::error::{DamlCodeGenError, DamlCodeGenResult};
use daml_lf::DarFile;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct DamlArchivePayload<'a> {
    pub name: &'a str,
    pub packages: HashMap<&'a str, DamlPackagePayload<'a>>,
}

impl<'a> DamlArchivePayload<'a> {
    pub fn package_by_id(&self, name: &str) -> Option<&DamlPackagePayload<'_>> {
        self.packages.get(name)
    }
}

impl<'a> TryFrom<&'a DarFile> for DamlArchivePayload<'a> {
    type Error = DamlCodeGenError;

    fn try_from(dar_file: &'a DarFile) -> DamlCodeGenResult<Self> {
        let name = &dar_file.main.name;
        let packages: HashMap<_, _> = dar_file
            .dependencies
            .iter()
            .chain(iter::once(&dar_file.main))
            .map(|p| Ok((p.hash.as_str(), DamlPackagePayload::try_from(p)?)))
            .collect::<DamlCodeGenResult<_>>()?;
        Ok(Self {
            name,
            packages,
        })
    }
}
