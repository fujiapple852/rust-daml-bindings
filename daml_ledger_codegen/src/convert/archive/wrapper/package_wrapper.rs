use crate::convert::archive::wrapper::payload::*;
use crate::convert::archive::wrapper::*;

#[derive(Debug, Clone, Copy)]
pub struct DamlPackageWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub payload: &'a DamlPackagePayload<'a>,
}

impl<'a> DamlPackageWrapper<'a> {
    pub fn wrap(parent_archive: &'a DamlArchivePayload, package: &'a DamlPackagePayload) -> Self {
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
