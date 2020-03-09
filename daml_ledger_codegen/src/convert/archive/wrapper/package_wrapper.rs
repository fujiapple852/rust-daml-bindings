use crate::convert::archive::wrapper::payload::{DamlArchivePayload, DamlPackagePayload};
use crate::convert::archive::wrapper::DamlModuleWrapper;

#[derive(Debug, Clone, Copy)]
pub struct DamlPackageWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub payload: &'a DamlPackagePayload<'a>,
}

impl<'a> DamlPackageWrapper<'a> {
    pub fn wrap(parent_archive: &'a DamlArchivePayload<'_>, package: &'a DamlPackagePayload<'_>) -> Self {
        Self {
            parent_archive,
            payload: package,
        }
    }

    pub fn modules(self) -> impl Iterator<Item = DamlModuleWrapper<'a>> {
        self.payload
            .modules
            .values()
            .map(move |module| DamlModuleWrapper::wrap(self.parent_archive, self.payload, module))
    }
}
