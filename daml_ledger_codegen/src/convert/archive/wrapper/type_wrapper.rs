use crate::convert::archive::wrapper::payload::{
    DamlArchivePayload, DamlDataPayload, DamlDataRefPayload, DamlModulePayload, DamlPackagePayload, DamlTypePayload,
    DamlVarPayload,
};
use crate::convert::archive::wrapper::DamlDataWrapper;
use crate::convert::error::{DamlCodeGenError, DamlCodeGenResult};

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
        parent_archive: &'a DamlArchivePayload<'_>,
        parent_package: &'a DamlPackagePayload<'_>,
        parent_module: &'a DamlModulePayload<'_>,
        parent_data: &'a DamlDataPayload<'_>,
        payload: &'a DamlTypePayload<'_>,
    ) -> Self {
        Self {
            parent_archive,
            parent_package,
            parent_module,
            parent_data,
            payload,
        }
    }

    pub fn nested_type(self) -> DamlCodeGenResult<DamlTypeWrapper<'a>> {
        match self.payload {
            DamlTypePayload::List(nested) | DamlTypePayload::TextMap(nested) | DamlTypePayload::Optional(nested) =>
                Ok(self.ty(nested)),
            _ => Err(DamlCodeGenError::UnexpectedType(
                "List, TextMap or Optional".to_owned(),
                self.payload.name_for_error().to_owned(),
            )),
        }
    }

    pub fn data_ref(self) -> DamlCodeGenResult<DamlDataRefWrapper<'a>> {
        match self.payload {
            DamlTypePayload::DataRef(data_ref) => Ok(DamlDataRefWrapper::new(
                self.parent_archive,
                self.parent_package,
                self.parent_module,
                self.parent_data,
                data_ref,
            )),
            _ => Err(DamlCodeGenError::UnexpectedType("DataRef".to_owned(), self.payload.name_for_error().to_owned())),
        }
    }

    pub fn contract_id_data_ref(self) -> DamlCodeGenResult<DamlDataRefWrapper<'a>> {
        match self.payload {
            DamlTypePayload::ContractId(Some(data_ref)) => Ok(DamlDataRefWrapper::new(
                self.parent_archive,
                self.parent_package,
                self.parent_module,
                self.parent_data,
                data_ref,
            )),
            _ => Err(DamlCodeGenError::UnexpectedType(
                "ContractId".to_string(),
                self.payload.name_for_error().to_owned(),
            )),
        }
    }

    pub fn var(self) -> DamlCodeGenResult<DamlVarWrapper<'a>> {
        match self.payload {
            DamlTypePayload::Var(var) => Ok(DamlVarWrapper::new(
                self.parent_archive,
                self.parent_package,
                self.parent_module,
                self.parent_data,
                var,
            )),
            _ => Err(DamlCodeGenError::UnexpectedType("Var".to_owned(), self.payload.name_for_error().to_owned())),
        }
    }

    fn ty(self, ty: &'a DamlTypePayload<'_>) -> DamlTypeWrapper<'a> {
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
        parent_archive: &'a DamlArchivePayload<'_>,
        parent_package: &'a DamlPackagePayload<'_>,
        parent_module: &'a DamlModulePayload<'_>,
        parent_data: &'a DamlDataPayload<'_>,
        payload: &'a DamlDataRefPayload<'_>,
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

    pub fn get_data(self) -> DamlCodeGenResult<DamlDataWrapper<'a>> {
        let source_resolver = self.parent_package;
        let target_package_id = self.payload.package_ref.resolve(source_resolver)?;
        let target_package: &DamlPackagePayload<'a> = self
            .parent_archive
            .package_by_id(target_package_id)
            .ok_or_else(|| DamlCodeGenError::UnknownPackage(target_package_id.to_owned()))?;
        let target_module = target_package
            .module_by_name(&self.payload.module_path.resolve(source_resolver)?.join("."))
            .ok_or_else(|| DamlCodeGenError::UnknownModule(self.payload.module_path.to_string()))?;
        let source_data_type_name = self.payload.data_name.resolve(source_resolver)?;
        let data_types_iter =
            target_module.data_types.iter().map(|dt| dt.name().resolve(target_package).map(|name| (name, dt)));
        let target_data_type = itertools::process_results(data_types_iter, |mut iter| {
            iter.find_map(|(name, dt)| {
                if name == source_data_type_name {
                    Some(dt)
                } else {
                    None
                }
            })
        })?
        .ok_or_else(|| DamlCodeGenError::UnknownData(source_data_type_name.join(".")))?;
        Ok(DamlDataWrapper::wrap(self.parent_archive, target_package, target_module, target_data_type))
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
        parent_archive: &'a DamlArchivePayload<'_>,
        parent_package: &'a DamlPackagePayload<'_>,
        parent_module: &'a DamlModulePayload<'_>,
        parent_data: &'a DamlDataPayload<'_>,
        payload: &'a DamlVarPayload<'_>,
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
