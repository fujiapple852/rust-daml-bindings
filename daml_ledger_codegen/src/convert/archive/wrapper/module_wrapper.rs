use crate::convert::archive::wrapper::payload::*;
use crate::convert::archive::wrapper::*;

#[derive(Debug, Clone, Copy)]
pub struct DamlModuleWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub parent_package: &'a DamlPackagePayload<'a>,
    pub payload: &'a DamlModulePayload<'a>,
}

impl<'a> DamlModuleWrapper<'a> {
    pub fn wrap(
        parent_archive: &'a DamlArchivePayload<'a>,
        parent_package: &'a DamlPackagePayload<'a>,
        payload: &'a DamlModulePayload<'a>,
    ) -> Self {
        Self {
            parent_archive,
            parent_package,
            payload,
        }
    }

    pub fn child_modules(self) -> impl Iterator<Item = DamlModuleWrapper<'a>> {
        self.payload.children.values().map(move |module| self.wrap_child_module(module))
    }

    pub fn data_types(self) -> impl Iterator<Item = DamlDataWrapper<'a>> {
        self.payload.data_types.values().map(move |dt| self.wrap_data_type(dt))
    }

    fn wrap_child_module(self, child_module: &'a DamlModulePayload) -> DamlModuleWrapper<'a> {
        DamlModuleWrapper::wrap(self.parent_archive, self.parent_package, child_module)
    }

    fn wrap_data_type(self, data: &'a DamlDataPayload) -> DamlDataWrapper<'a> {
        DamlDataWrapper::wrap(self.parent_archive, self.parent_package, self.payload, data)
    }
}
