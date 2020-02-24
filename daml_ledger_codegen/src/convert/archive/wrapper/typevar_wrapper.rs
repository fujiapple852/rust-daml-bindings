use crate::convert::archive::wrapper::payload::*;

#[derive(Debug, Clone, Copy)]
pub struct DamlTypeVarWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub parent_package: &'a DamlPackagePayload<'a>,
    pub parent_module: &'a DamlModulePayload<'a>,
    pub parent_data: &'a DamlDataPayload<'a>,
    pub payload: &'a DamlTypeVarPayload<'a>,
}

impl<'a> DamlTypeVarWrapper<'a> {
    pub fn wrap(
        parent_archive: &'a DamlArchivePayload<'a>,
        parent_package: &'a DamlPackagePayload<'a>,
        parent_module: &'a DamlModulePayload<'a>,
        parent_data: &'a DamlDataPayload<'a>,
        payload: &'a DamlTypeVarPayload<'a>,
    ) -> Self {
        Self {
            parent_archive,
            parent_package,
            parent_module,
            parent_data,
            payload,
        }
    }
}
