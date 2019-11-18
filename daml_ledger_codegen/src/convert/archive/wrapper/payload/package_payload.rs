use crate::convert::archive::wrapper::payload::*;
use daml_lf::{DamlLfArchive, DamlLfPackage};

#[derive(Debug)]
pub struct DamlPackagePayload<'a> {
    pub name: &'a str,
    pub package_id: &'a str,
    pub interned_package_ids: &'a [String],
    pub root_module: DamlModulePayload<'a>,
}

impl<'a> PartialEq for DamlPackagePayload<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<'a> DamlPackagePayload<'a> {
    #[allow(clippy::cast_possible_truncation)]
    pub fn lookup_package_id_by_ref(&self, package_ref: &'a DamlPackageRef) -> Option<&str> {
        match package_ref {
            DamlPackageRef::This => Some(self.package_id),
            &DamlPackageRef::PackageId(s) => Some(s),
            &DamlPackageRef::InternedId(i) => self.interned_package_ids.get(i as usize).map(AsRef::as_ref),
        }
    }

    pub fn module_by_name(&self, module_path: &'a [String]) -> Option<&DamlModulePayload> {
        self.root_module.module_by_path(module_path)
    }
}

impl<'a> From<&'a DamlLfArchive> for DamlPackagePayload<'a> {
    fn from(daml_lf_archive: &'a DamlLfArchive) -> Self {
        match &daml_lf_archive.payload.package {
            DamlLfPackage::V1(p) => Self {
                name: daml_lf_archive.name.as_str(),
                package_id: daml_lf_archive.hash.as_str(),
                interned_package_ids: p.interned_package_ids.as_slice(),
                root_module: DamlModulePayload::from_modules(p.modules.as_slice()),
            },
        }
    }
}
