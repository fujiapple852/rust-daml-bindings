use crate::convert::archive::wrapper::payload::{
    DamlArchivePayload, DamlDataPayload, DamlFieldPayload, DamlModulePayload, DamlPackagePayload,
};
use crate::convert::archive::wrapper::DamlTypeWrapper;

#[derive(Debug, Clone, Copy)]
pub struct DamlFieldWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub parent_package: &'a DamlPackagePayload<'a>,
    pub parent_module: &'a DamlModulePayload<'a>,
    pub parent_data: &'a DamlDataPayload<'a>,
    pub payload: &'a DamlFieldPayload<'a>,
}

impl<'a> DamlFieldWrapper<'a> {
    pub fn wrap(
        parent_archive: &'a DamlArchivePayload<'a>,
        parent_package: &'a DamlPackagePayload<'a>,
        parent_module: &'a DamlModulePayload<'a>,
        parent_data: &'a DamlDataPayload<'a>,
        payload: &'a DamlFieldPayload<'a>,
    ) -> Self {
        Self {
            parent_archive,
            parent_package,
            parent_module,
            parent_data,
            payload,
        }
    }

    pub fn ty(self) -> DamlTypeWrapper<'a> {
        DamlTypeWrapper {
            parent_archive: self.parent_archive,
            parent_package: self.parent_package,
            parent_module: self.parent_module,
            parent_data: self.parent_data,
            payload: &self.payload.ty,
        }
    }
}
