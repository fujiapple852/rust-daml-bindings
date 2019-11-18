use crate::convert::archive::wrapper::payload::*;
use crate::convert::archive::wrapper::*;

#[derive(Debug, Clone, Copy)]
pub struct DamlTypeWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub parent_package: &'a DamlPackagePayload<'a>,
    pub parent_module: &'a DamlModulePayload<'a>,
    pub parent_data: &'a DamlDataPayload<'a>,
    pub payload: &'a DamlTypePayload<'a>,
}

impl<'a> DamlTypeWrapper<'a> {
    pub fn wrap(
        parent_archive: &'a DamlArchivePayload,
        parent_package: &'a DamlPackagePayload,
        parent_module: &'a DamlModulePayload,
        parent_data: &'a DamlDataPayload,
        payload: &'a DamlTypePayload,
    ) -> Self {
        Self {
            parent_archive,
            parent_package,
            parent_module,
            parent_data,
            payload,
        }
    }

    pub fn nested_type(self) -> DamlTypeWrapper<'a> {
        match self.payload {
            DamlTypePayload::List(nested) | DamlTypePayload::TextMap(nested) | DamlTypePayload::Optional(nested) =>
                self.ty(nested),
            _ => panic!("expected List, TextMap, Optional or ContractId"),
        }
    }

    pub fn data_ref(self) -> DamlDataRefWrapper<'a> {
        match self.payload {
            DamlTypePayload::DataRef(data_ref) =>
                DamlDataRefWrapper::new(self.parent_archive, self.parent_package, self.parent_module, data_ref),
            _ => panic!("expected DataRef"),
        }
    }

    pub fn contract_id_data_ref(self) -> DamlDataRefWrapper<'a> {
        match self.payload {
            DamlTypePayload::ContractId(Some(data_ref)) =>
                DamlDataRefWrapper::new(self.parent_archive, self.parent_package, self.parent_module, data_ref),
            _ => panic!("expected ContractId"),
        }
    }

    fn ty(self, ty: &'a DamlTypePayload) -> DamlTypeWrapper<'a> {
        Self {
            payload: ty,
            ..self
        }
    }

    pub fn parent_data(self) -> DamlDataWrapper<'a> {
        DamlDataWrapper::wrap(self.parent_archive, self.parent_package, self.parent_module, self.parent_data)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamlDataRefWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub parent_package: &'a DamlPackagePayload<'a>,
    pub parent_module: &'a DamlModulePayload<'a>,
    pub payload: &'a DamlDataRefPayload<'a>,
}

impl<'a> DamlDataRefWrapper<'a> {
    pub fn new(
        parent_archive: &'a DamlArchivePayload,
        parent_package: &'a DamlPackagePayload,
        parent_module: &'a DamlModulePayload,
        payload: &'a DamlDataRefPayload,
    ) -> Self {
        Self {
            parent_archive,
            parent_package,
            parent_module,
            payload,
        }
    }

    pub fn get_data(self) -> DamlDataWrapper<'a> {
        let target_package_id = self
            .parent_package
            .lookup_package_id_by_ref(&self.payload.package_ref)
            .expect("lookup_package_id_by_ref failed");
        let target_package = self
            .parent_archive
            .package_by_id(target_package_id)
            .unwrap_or_else(|| panic!("package_by_id lookup failed for {}", target_package_id));
        let target_module = target_package
            .module_by_name(&self.payload.module_path)
            .unwrap_or_else(|| panic!("module_by_name lookup failed for {}", &self.payload.module_path.join(".")));
        let target_data_type = target_module
            .data_type(&self.payload.data_name)
            .unwrap_or_else(|| panic!("data_type lookup failed for {}", &self.payload.data_name));
        DamlDataWrapper::wrap(self.parent_archive, target_package, target_module, target_data_type)
    }
}
