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
            DamlTypePayload::DataRef(data_ref) => DamlDataRefWrapper::new(
                self.parent_archive,
                self.parent_package,
                self.parent_module,
                self.parent_data,
                data_ref,
            ),
            _ => panic!("expected DataRef"),
        }
    }

    pub fn contract_id_data_ref(self) -> DamlDataRefWrapper<'a> {
        match self.payload {
            DamlTypePayload::ContractId(Some(data_ref)) => DamlDataRefWrapper::new(
                self.parent_archive,
                self.parent_package,
                self.parent_module,
                self.parent_data,
                data_ref,
            ),
            _ => panic!("expected ContractId"),
        }
    }

    pub fn var(self) -> DamlVarWrapper<'a> {
        match self.payload {
            DamlTypePayload::Var(var) =>
                DamlVarWrapper::new(self.parent_archive, self.parent_package, self.parent_module, self.parent_data, var),
            _ => panic!("expected Var"),
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

#[derive(Debug, Clone, Copy)]
pub struct DamlDataRefWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub parent_package: &'a DamlPackagePayload<'a>,
    pub parent_module: &'a DamlModulePayload<'a>,
    pub parent_data: &'a DamlDataPayload<'a>,
    pub payload: &'a DamlDataRefPayload<'a>,
}

impl<'a> DamlDataRefWrapper<'a> {
    pub fn new(
        parent_archive: &'a DamlArchivePayload,
        parent_package: &'a DamlPackagePayload,
        parent_module: &'a DamlModulePayload,
        parent_data: &'a DamlDataPayload,
        payload: &'a DamlDataRefPayload,
    ) -> Self {
        Self {
            parent_archive,
            parent_package,
            parent_module,
            parent_data,
            payload,
        }
    }

    pub fn type_arguments(self) -> impl Iterator<Item = DamlTypeWrapper<'a>> {
        self.payload.type_arguments.iter().map(move |arg| {
            DamlTypeWrapper::wrap(self.parent_archive, self.parent_package, self.parent_module, self.parent_data, arg)
        })
    }

    pub fn get_data(self) -> DamlDataWrapper<'a> {
        let source_resolver = self.parent_package;
        let target_package_id =
            self.payload.package_ref.resolve(source_resolver).expect("lookup_package_id_by_ref failed");
        let target_package = self
            .parent_archive
            .package_by_id(target_package_id)
            .unwrap_or_else(|| panic!("package_by_id lookup failed for {}", target_package_id));
        let target_module = target_package
            .module_by_name(&self.payload.module_path.resolve(source_resolver).join("."))
            .unwrap_or_else(|| panic!("module_by_name lookup failed for {}", &self.payload.module_path.to_string()));
        let source_data_type_name = self.payload.data_name.resolve(source_resolver);
        let target_data_type = target_module
            .data_types
            .iter()
            .find(|dt| dt.name().resolve(target_package) == source_data_type_name)
            .unwrap_or_else(|| panic!("data_type lookup failed for {}", &self.payload.data_name.to_string()));
        DamlDataWrapper::wrap(self.parent_archive, target_package, target_module, target_data_type)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DamlVarWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub parent_package: &'a DamlPackagePayload<'a>,
    pub parent_module: &'a DamlModulePayload<'a>,
    pub parent_data: &'a DamlDataPayload<'a>,
    pub payload: &'a DamlVarPayload<'a>,
}

impl<'a> DamlVarWrapper<'a> {
    pub fn new(
        parent_archive: &'a DamlArchivePayload,
        parent_package: &'a DamlPackagePayload,
        parent_module: &'a DamlModulePayload,
        parent_data: &'a DamlDataPayload,
        payload: &'a DamlVarPayload,
    ) -> Self {
        Self {
            parent_archive,
            parent_package,
            parent_module,
            parent_data,
            payload,
        }
    }

    pub fn type_arguments(self) -> impl Iterator<Item = DamlTypeWrapper<'a>> {
        self.payload.type_arguments.iter().map(move |arg| {
            DamlTypeWrapper::wrap(self.parent_archive, self.parent_package, self.parent_module, self.parent_data, arg)
        })
    }
}
