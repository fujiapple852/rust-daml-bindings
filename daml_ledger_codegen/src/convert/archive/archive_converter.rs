use crate::convert::archive::wrapper::DamlTypePayload;
use crate::convert::archive::wrapper::PackageInternedResolver;
use crate::convert::archive::wrapper::{
    DamlArchiveWrapper, DamlChoiceWrapper, DamlDataBoxChecker, DamlDataRefWrapper, DamlDataWrapper, DamlFieldWrapper,
    DamlKindPayload, DamlModuleWrapper, DamlPackageWrapper, DamlTypeVarWrapper, DamlTypeWrapper, DamlVarWrapper,
};
use crate::convert::error::{DamlCodeGenError, DamlCodeGenResult};
use crate::element::{
    DamlArchive, DamlArrow, DamlChoice, DamlData, DamlDataRef, DamlEnum, DamlField, DamlKind, DamlLocalDataRef,
    DamlModule, DamlNonLocalDataRef, DamlPackage, DamlRecord, DamlTemplate, DamlType, DamlTypeVar, DamlVar,
    DamlVariant,
};
use daml_lf::LanguageFeatureVersion;
use std::collections::HashMap;
use std::convert::TryFrom;

/// Convert from `DamlArchiveWrapper` to `DamlArchive`.
impl<'a> TryFrom<&DamlArchiveWrapper<'a>> for DamlArchive<'a> {
    type Error = DamlCodeGenError;

    fn try_from(archive: &DamlArchiveWrapper<'a>) -> DamlCodeGenResult<Self> {
        let packages: HashMap<_, _> = archive
            .packages()
            .map(|v| Ok((v.payload.name.as_str(), DamlPackage::try_from(&v)?)))
            .collect::<DamlCodeGenResult<_>>()?;
        Ok(DamlArchive::new(archive.payload.name, packages))
    }
}

/// Convert from `DamlPackageWrapper` to `DamlPackage`.
impl<'a> TryFrom<&DamlPackageWrapper<'a>> for DamlPackage<'a> {
    type Error = DamlCodeGenError;

    fn try_from(package: &DamlPackageWrapper<'a>) -> DamlCodeGenResult<Self> {
        fn from_modules<'a, T: Iterator<Item = DamlModuleWrapper<'a>>>(
            modules: T,
        ) -> DamlCodeGenResult<DamlModule<'a>> {
            Ok(modules.fold(Ok(DamlModule::new_root()), |mut root, module| {
                if let Ok(r) = root.as_mut() {
                    let path = module.payload.path.resolve(module.parent_package)?;
                    let data_types: Vec<_> =
                        module.data_types().map(|dt| DamlData::try_from(&dt)).collect::<DamlCodeGenResult<_>>()?;
                    add_module_to_tree(r, data_types, path.clone(), path);
                }
                root
            })?)
        }

        fn add_module_to_tree<'a>(
            node: &mut DamlModule<'a>,
            data_types: Vec<DamlData<'a>>,
            full_path: Vec<&'a str>,
            remaining_path: Vec<&'a str>,
        ) {
            if let Some(&child_mod_name) = remaining_path.first() {
                let child_mod_path = &full_path[..=full_path.len() - remaining_path.len()];
                let entry = node
                    .child_modules
                    .entry(child_mod_name)
                    .or_insert_with(|| DamlModule::new(child_mod_path.to_vec()));
                add_module_to_tree(entry, data_types, full_path, remaining_path[1..].to_vec())
            } else {
                node.data_types = data_types.into_iter().map(|dt| (dt.name(), dt)).collect();
            }
        }
        Ok(DamlPackage::new(
            &package.payload.name,
            package.payload.package_id,
            package.payload.version.as_ref().map(AsRef::as_ref),
            from_modules(package.modules())?,
        ))
    }
}

/// Convert from `DamlDataWrapper` to `DamlData`.
impl<'a> TryFrom<&DamlDataWrapper<'a>> for DamlData<'a> {
    type Error = DamlCodeGenError;

    fn try_from(data: &DamlDataWrapper<'a>) -> DamlCodeGenResult<Self> {
        Ok(match data {
            DamlDataWrapper::Record(record) => {
                let name = record.payload.name.resolve_last(record.parent_package)?;
                let type_arguments: Vec<_> =
                    record.type_arguments().map(DamlTypeVar::try_from).collect::<DamlCodeGenResult<_>>()?;
                let fields: Vec<_> =
                    record.fields().map(|f| DamlField::try_from(&f)).collect::<DamlCodeGenResult<_>>()?;
                DamlData::Record(DamlRecord::new(name, fields, type_arguments))
            },
            DamlDataWrapper::Template(template) => {
                let resolver = template.parent_package;
                let name = template.payload.name.resolve_last(template.parent_package)?;
                let module_path = template.parent_module.path.resolve(resolver)?;
                let fields: Vec<_> =
                    template.fields()?.map(|f| DamlField::try_from(&f)).collect::<DamlCodeGenResult<_>>()?;
                let choices: Vec<_> =
                    template.choices().map(|c| DamlChoice::try_from(&c)).collect::<DamlCodeGenResult<_>>()?;
                DamlData::Template(DamlTemplate::new(
                    name,
                    template.parent_package.package_id,
                    module_path,
                    fields,
                    choices,
                ))
            },
            DamlDataWrapper::Variant(variant) => {
                let name = variant.payload.name.resolve_last(variant.parent_package)?;
                let type_arguments: Vec<_> =
                    variant.type_arguments().map(DamlTypeVar::try_from).collect::<DamlCodeGenResult<_>>()?;
                let fields: Vec<_> =
                    variant.fields().map(|f| DamlField::try_from(&f)).collect::<DamlCodeGenResult<_>>()?;
                DamlData::Variant(DamlVariant::new(name, fields, type_arguments))
            },
            DamlDataWrapper::Enum(data_enum) => {
                let resolver = data_enum.parent_package;
                let name = data_enum.payload.name.resolve_last(resolver)?;
                let type_arguments: Vec<_> =
                    data_enum.type_arguments().map(DamlTypeVar::try_from).collect::<DamlCodeGenResult<_>>()?;
                let constructors: Vec<&str> = if data_enum
                    .parent_package
                    .language_version
                    .supports_feature(&LanguageFeatureVersion::INTERNED_STRINGS)
                {
                    assert!(data_enum.payload.constructors_str.is_empty(), "constructors_str should be empty!");
                    resolver.resolve_strings(data_enum.payload.constructors_interned_str)?
                } else {
                    assert!(
                        data_enum.payload.constructors_interned_str.is_empty(),
                        "constructors_interned_str should be empty!"
                    );
                    data_enum.payload.constructors_str.iter().map(AsRef::as_ref).collect()
                };
                DamlData::Enum(DamlEnum::new(name, constructors, type_arguments))
            },
        })
    }
}

/// Convert from `DamlChoiceWrapper` to `DamlChoice`.
impl<'a> TryFrom<&DamlChoiceWrapper<'a>> for DamlChoice<'a> {
    type Error = DamlCodeGenError;

    fn try_from(choice: &DamlChoiceWrapper<'a>) -> DamlCodeGenResult<Self> {
        let name = choice.payload.name.resolve(choice.parent_package)?;
        let target_data_type = choice.argument_type().data_ref()?.get_data()?;
        match target_data_type {
            DamlDataWrapper::Record(record) => {
                let fields: Vec<_> =
                    record.fields().map(|f| DamlField::try_from(&f)).collect::<DamlCodeGenResult<_>>()?;
                Ok(DamlChoice::new(name, fields, DamlType::try_from(&choice.return_type())?))
            },
            _ => Err(DamlCodeGenError::UnexpectedChoiceData),
        }
    }
}

/// Convert from `DamlFieldWrapper` to `DamlField`.
impl<'a> TryFrom<&DamlFieldWrapper<'a>> for DamlField<'a> {
    type Error = DamlCodeGenError;

    fn try_from(field: &DamlFieldWrapper<'a>) -> DamlCodeGenResult<Self> {
        Ok(DamlField::new(field.payload.name.resolve(field.parent_package)?, DamlType::try_from(&field.ty())?))
    }
}

/// Convert from `DamlTypeWrapper` to `DamlType`.
impl<'a> TryFrom<&DamlTypeWrapper<'a>> for DamlType<'a> {
    type Error = DamlCodeGenError;

    fn try_from(daml_type: &DamlTypeWrapper<'a>) -> DamlCodeGenResult<Self> {
        fn make_data_ref<'a>(
            daml_type: &DamlTypeWrapper<'a>,
            data_ref: DamlDataRefWrapper<'a>,
            target_data: DamlDataWrapper<'a>,
        ) -> DamlCodeGenResult<DamlDataRef<'a>> {
            let resolver = daml_type.parent_package;
            let current_package_name = daml_type.parent_package.name.as_str();
            let target_package_name = match target_data {
                DamlDataWrapper::Record(record) => record.parent_package.name.as_str(),
                DamlDataWrapper::Template(template) => template.parent_package.name.as_str(),
                DamlDataWrapper::Variant(variant) => variant.parent_package.name.as_str(),
                DamlDataWrapper::Enum(data_enum) => data_enum.parent_package.name.as_str(),
            };
            let current_module_path = daml_type.parent_module.path.resolve(resolver)?;
            let target_module_path = data_ref.payload.module_path.resolve(resolver)?;
            let data_name = data_ref.payload.data_name.resolve_last(data_ref.parent_package)?;
            let type_arguments: Vec<_> =
                data_ref.type_arguments().map(|arg| DamlType::try_from(&arg)).collect::<DamlCodeGenResult<_>>()?;
            if target_package_name == current_package_name && target_module_path == current_module_path {
                Ok(DamlDataRef::Local(DamlLocalDataRef::new(
                    data_name,
                    target_package_name,
                    target_module_path,
                    type_arguments,
                )))
            } else {
                Ok(DamlDataRef::NonLocal(DamlNonLocalDataRef::new(
                    data_name,
                    current_package_name,
                    current_module_path,
                    target_package_name,
                    target_module_path,
                    type_arguments,
                )))
            }
        }

        Ok(match daml_type.payload {
            DamlTypePayload::ContractId(Some(_)) => {
                let data_ref_wrapper = daml_type.contract_id_data_ref()?;
                let target_data_wrapper = data_ref_wrapper.get_data()?;
                DamlType::ContractId(Some(make_data_ref(daml_type, data_ref_wrapper, target_data_wrapper)?))
            },
            DamlTypePayload::ContractId(None) => DamlType::ContractId(None),
            DamlTypePayload::Int64 => DamlType::Int64,
            DamlTypePayload::Numeric => DamlType::Numeric,
            DamlTypePayload::Text => DamlType::Text,
            DamlTypePayload::Timestamp => DamlType::Timestamp,
            DamlTypePayload::Party => DamlType::Party,
            DamlTypePayload::Bool => DamlType::Bool,
            DamlTypePayload::Unit => DamlType::Unit,
            DamlTypePayload::Date => DamlType::Date,
            DamlTypePayload::List(_) => DamlType::List(Box::new(DamlType::try_from(&daml_type.nested_type()?)?)),
            DamlTypePayload::Update => DamlType::Update,
            DamlTypePayload::Scenario => DamlType::Scenario,
            DamlTypePayload::TextMap(_) => DamlType::TextMap(Box::new(DamlType::try_from(&daml_type.nested_type()?)?)),
            DamlTypePayload::Optional(_) =>
                DamlType::Optional(Box::new(DamlType::try_from(&daml_type.nested_type()?)?)),
            DamlTypePayload::DataRef(_) => {
                let data_ref_wrapper = daml_type.data_ref()?;
                let target_data_wrapper = data_ref_wrapper.get_data()?;
                let data_ref = make_data_ref(daml_type, data_ref_wrapper, target_data_wrapper)?;
                if DamlDataBoxChecker::should_box(daml_type.parent_data(), target_data_wrapper)? {
                    DamlType::BoxedDataRef(data_ref)
                } else {
                    DamlType::DataRef(data_ref)
                }
            },
            DamlTypePayload::Var(_) => DamlType::Var(DamlVar::try_from(&daml_type.var()?)?),
            DamlTypePayload::Arrow => DamlType::Arrow,
            DamlTypePayload::Any => DamlType::Any,
            DamlTypePayload::TypeRep => DamlType::TypeRep,
        })
    }
}

/// Convert from `DamlTypeVarWrapper` to `DamlTypeVar`.
impl<'a> TryFrom<DamlTypeVarWrapper<'a>> for DamlTypeVar<'a> {
    type Error = DamlCodeGenError;

    fn try_from(typevar: DamlTypeVarWrapper<'a>) -> DamlCodeGenResult<Self> {
        Ok(DamlTypeVar::new(
            typevar.payload.var.resolve(typevar.parent_package)?,
            DamlKind::from(&typevar.payload.kind),
        ))
    }
}

/// Convert from `DamlVarWrapper` to `DamlVar`.
impl<'a> TryFrom<&DamlVarWrapper<'a>> for DamlVar<'a> {
    type Error = DamlCodeGenError;

    fn try_from(var: &DamlVarWrapper<'a>) -> DamlCodeGenResult<Self> {
        let type_arguments =
            var.type_arguments().map(|arg| DamlType::try_from(&arg)).collect::<DamlCodeGenResult<_>>()?;
        Ok(DamlVar::new(var.payload.var.resolve(var.parent_package)?, type_arguments))
    }
}

/// Convert from `DamlKindPayload` to `DamlKind`.
impl From<&DamlKindPayload> for DamlKind {
    fn from(kind: &DamlKindPayload) -> Self {
        match kind {
            DamlKindPayload::Star => DamlKind::Star,
            DamlKindPayload::Arrow(arrow) => DamlKind::Arrow(Box::new(DamlArrow::new(
                arrow.params.iter().map(DamlKind::from).collect(),
                DamlKind::from(&arrow.result),
            ))),
            DamlKindPayload::Nat => DamlKind::Nat,
        }
    }
}
