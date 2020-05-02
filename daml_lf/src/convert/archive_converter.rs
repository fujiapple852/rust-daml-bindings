use crate::convert::archive_payload::DamlArchiveWrapper;
use crate::convert::data_data_box_checker::DamlDataBoxChecker;
use crate::convert::data_payload::{DamlChoiceWrapper, DamlDataEnrichedPayload, DamlDataPayload, DamlDataWrapper};
use crate::convert::field_payload::{DamlFieldPayload, DamlFieldWrapper};
use crate::convert::interned::PackageInternedResolver;
use crate::convert::module_payload::DamlModuleWrapper;
use crate::convert::package_payload::DamlPackageWrapper;
use crate::convert::resolver::resolve_data_ref;
use crate::convert::type_payload::{DamlDataRefWrapper, DamlTypePayload, DamlTypeWrapper, DamlVarWrapper};
use crate::convert::typevar_payload::{DamlKindPayload, DamlTypeVarPayload, DamlTypeVarWrapper};
use crate::convert::wrapper::DamlPayloadDataWrapper;
use crate::element::{
    DamlArchive, DamlArrow, DamlChoice, DamlData, DamlDataRef, DamlEnum, DamlField, DamlKind, DamlLocalDataRef,
    DamlModule, DamlNonLocalDataRef, DamlPackage, DamlRecord, DamlTemplate, DamlType, DamlTypeVar, DamlVar,
    DamlVariant,
};
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::LanguageFeatureVersion;
use std::collections::HashMap;
use std::convert::TryFrom;

/// Convert from `DamlArchiveWrapper` to `DamlArchive`.
impl<'a> TryFrom<DamlArchiveWrapper<'a>> for DamlArchive<'a> {
    type Error = DamlLfConvertError;

    fn try_from(archive: DamlArchiveWrapper<'a>) -> DamlLfConvertResult<Self> {
        let packages: HashMap<_, _> = archive
            .archive
            .packages
            .values()
            .map(|package| Ok((package.name.as_str(), DamlPackage::try_from(archive.with_package(package))?)))
            .collect::<DamlLfConvertResult<_>>()?;
        Ok(DamlArchive::new(archive.archive.name, packages))
    }
}

/// Convert from `DamlPackageWrapper` to `DamlPackage`.
impl<'a> TryFrom<DamlPackageWrapper<'a>> for DamlPackage<'a> {
    type Error = DamlLfConvertError;

    fn try_from(package: DamlPackageWrapper<'a>) -> DamlLfConvertResult<Self> {
        fn from_modules<'a, T: Iterator<Item = DamlModuleWrapper<'a>>>(
            modules: T,
        ) -> DamlLfConvertResult<DamlModule<'a>> {
            Ok(modules.fold(Ok(DamlModule::new_root()), |mut root, module| {
                if let Ok(r) = root.as_mut() {
                    let path = module.module.path.resolve(module.package)?;
                    let data_types: Vec<_> = module
                        .module
                        .data_types
                        .iter()
                        .map(|dt| DamlData::try_from(enriched_data(module.with_data(dt))))
                        .collect::<DamlLfConvertResult<_>>()?;
                    add_module_to_tree(r, data_types, path.clone(), &path);
                }
                root
            })?)
        }

        fn add_module_to_tree<'a>(
            node: &mut DamlModule<'a>,
            data_types: Vec<DamlData<'a>>,
            full_path: Vec<&'a str>,
            remaining_path: &[&'a str],
        ) {
            if let Some(&child_mod_name) = remaining_path.first() {
                let child_mod_path = &full_path[..=full_path.len() - remaining_path.len()];
                let entry = node
                    .child_modules
                    .entry(child_mod_name)
                    .or_insert_with(|| DamlModule::new(child_mod_path.to_vec()));
                add_module_to_tree(entry, data_types, full_path, &remaining_path[1..])
            } else {
                node.data_types = data_types.into_iter().map(|dt| (dt.name(), dt)).collect();
            }
        }
        Ok(DamlPackage::new(
            &package.package.name,
            package.package.package_id,
            package.package.version.as_ref().map(AsRef::as_ref),
            package.package.language_version,
            from_modules(package.package.modules.values().map(|module| package.with_module(module)))?,
        ))
    }
}

/// Convert from `DamlDataWrapper` to `DamlData`.
impl<'a> TryFrom<DamlDataWrapper<'a>> for DamlData<'a> {
    type Error = DamlLfConvertError;

    fn try_from(data: DamlDataWrapper<'a>) -> DamlLfConvertResult<Self> {
        fn convert_fields<'a>(
            data: DamlDataWrapper<'a>,
            fields: &'a [DamlFieldPayload<'a>],
        ) -> DamlLfConvertResult<Vec<DamlField<'a>>> {
            fields.iter().map(|field| DamlField::try_from(data.wrap(field))).collect::<DamlLfConvertResult<_>>()
        }
        fn convert_type_arguments<'a>(
            data: DamlDataWrapper<'a>,
            type_arguments: &'a [DamlTypeVarPayload<'a>],
        ) -> DamlLfConvertResult<Vec<DamlTypeVar<'a>>> {
            type_arguments
                .iter()
                .map(|ty_arg| DamlTypeVar::try_from(data.wrap(ty_arg)))
                .collect::<DamlLfConvertResult<_>>()
        }
        let resolver = data.context.package;
        Ok(match data.payload {
            DamlDataEnrichedPayload::Record(record) => {
                let name = record.name.resolve_last(resolver)?;
                let type_arguments = convert_type_arguments(data, &record.type_arguments)?;
                let fields = convert_fields(data, &record.fields)?;
                DamlData::Record(DamlRecord::new(name, fields, type_arguments))
            },
            DamlDataEnrichedPayload::Template(template) => {
                let name = template.name.resolve_last(resolver)?;
                let module_path = data.context.module.path.resolve(resolver)?;
                let parent_data = match data.context.data {
                    DamlDataPayload::Record(record) => Ok(record),
                    _ => Err(DamlLfConvertError::UnexpectedData),
                }?;
                let fields = convert_fields(data, &parent_data.fields)?;
                let choices: Vec<_> = template
                    .choices
                    .iter()
                    .map(|choice| DamlChoice::try_from(data.wrap(choice)))
                    .collect::<DamlLfConvertResult<_>>()?;
                DamlData::Template(DamlTemplate::new(
                    name,
                    data.context.package.package_id,
                    module_path,
                    fields,
                    choices,
                ))
            },
            DamlDataEnrichedPayload::Variant(variant) => {
                let name = variant.name.resolve_last(resolver)?;
                let type_arguments = convert_type_arguments(data, &variant.type_arguments)?;
                let fields = convert_fields(data, &variant.fields)?;
                DamlData::Variant(DamlVariant::new(name, fields, type_arguments))
            },
            DamlDataEnrichedPayload::Enum(data_enum) => {
                let name = data_enum.name.resolve_last(resolver)?;
                let type_arguments = convert_type_arguments(data, &data_enum.type_arguments)?;
                let constructors: Vec<&str> = if data
                    .context
                    .package
                    .language_version
                    .supports_feature(&LanguageFeatureVersion::INTERNED_STRINGS)
                {
                    assert!(data_enum.constructors_str.is_empty(), "constructors_str should be empty!");
                    data.context.package.resolve_strings(data_enum.constructors_interned_str)?
                } else {
                    assert!(
                        data_enum.constructors_interned_str.is_empty(),
                        "constructors_interned_str should be empty!"
                    );
                    data_enum.constructors_str.iter().map(AsRef::as_ref).collect()
                };
                DamlData::Enum(DamlEnum::new(name, constructors, type_arguments))
            },
        })
    }
}

/// Convert from `DamlChoiceWrapper` to `DamlChoice`.
impl<'a> TryFrom<DamlChoiceWrapper<'a>> for DamlChoice<'a> {
    type Error = DamlLfConvertError;

    fn try_from(choice: DamlChoiceWrapper<'a>) -> DamlLfConvertResult<Self> {
        let name = choice.payload.name.resolve(choice.context.package)?;
        let target_data_wrapper = match &choice.payload.argument_type {
            DamlTypePayload::DataRef(data_ref) => Ok(resolve_data_ref(choice.wrap(data_ref))?),
            _ => Err(DamlLfConvertError::UnexpectedType(
                "DataRef".to_owned(),
                choice.payload.argument_type.name_for_error().to_owned(),
            )),
        }?;
        let fields = match target_data_wrapper.payload {
            DamlDataEnrichedPayload::Record(record) => Ok(record
                .fields
                .iter()
                .map(|field| DamlField::try_from(target_data_wrapper.wrap(field)))
                .collect::<DamlLfConvertResult<_>>()?),
            _ => Err(DamlLfConvertError::UnexpectedChoiceData),
        }?;
        let return_type = DamlType::try_from(&choice.wrap(&choice.payload.return_type))?;
        Ok(DamlChoice::new(name, fields, return_type))
    }
}

/// Convert from `DamlFieldWrapper` to `DamlField`.
impl<'a> TryFrom<DamlFieldWrapper<'a>> for DamlField<'a> {
    type Error = DamlLfConvertError;

    fn try_from(field: DamlFieldWrapper<'a>) -> DamlLfConvertResult<Self> {
        Ok(DamlField::new(
            field.payload.name.resolve(field.context.package)?,
            DamlType::try_from(&field.wrap(&field.payload.ty))?,
        ))
    }
}

/// Convert from `DamlTypeWrapper` to `DamlType`.
impl<'a> TryFrom<&DamlTypeWrapper<'a>> for DamlType<'a> {
    type Error = DamlLfConvertError;

    fn try_from(daml_type: &DamlTypeWrapper<'a>) -> Result<Self, Self::Error> {
        Ok(match daml_type.payload {
            DamlTypePayload::ContractId(Some(data_ref)) => {
                let data_ref_wrapper = daml_type.wrap(data_ref);
                let target_data_wrapper = resolve_data_ref(data_ref_wrapper)?;
                DamlType::ContractId(Some(make_data_ref(data_ref_wrapper, target_data_wrapper)?))
            },
            DamlTypePayload::ContractId(None) => DamlType::ContractId(None),
            DamlTypePayload::Int64 => DamlType::Int64,
            DamlTypePayload::Numeric(inner_type) =>
                DamlType::Numeric(Box::new(DamlType::try_from(&daml_type.wrap(inner_type.as_ref()))?)),
            DamlTypePayload::Text => DamlType::Text,
            DamlTypePayload::Timestamp => DamlType::Timestamp,
            DamlTypePayload::Party => DamlType::Party,
            DamlTypePayload::Bool => DamlType::Bool,
            DamlTypePayload::Unit => DamlType::Unit,
            DamlTypePayload::Date => DamlType::Date,
            DamlTypePayload::List(inner_type) =>
                DamlType::List(Box::new(DamlType::try_from(&daml_type.wrap(inner_type.as_ref()))?)),
            DamlTypePayload::Update => DamlType::Update,
            DamlTypePayload::Scenario => DamlType::Scenario,
            DamlTypePayload::TextMap(inner_type) =>
                DamlType::TextMap(Box::new(DamlType::try_from(&daml_type.wrap(inner_type.as_ref()))?)),
            DamlTypePayload::Optional(inner_type) =>
                DamlType::Optional(Box::new(DamlType::try_from(&daml_type.wrap(inner_type.as_ref()))?)),
            DamlTypePayload::DataRef(data_ref) => {
                let data_ref_wrapper = daml_type.wrap(data_ref);
                let target_data_wrapper = resolve_data_ref(data_ref_wrapper)?;
                let data_ref = make_data_ref(data_ref_wrapper, target_data_wrapper)?;
                if DamlDataBoxChecker::should_box(enriched_data(daml_type.context), target_data_wrapper)? {
                    DamlType::BoxedDataRef(data_ref)
                } else {
                    DamlType::DataRef(data_ref)
                }
            },
            DamlTypePayload::Var(var) => DamlType::Var(DamlVar::try_from(&daml_type.wrap(var))?),
            DamlTypePayload::Arrow => DamlType::Arrow,
            DamlTypePayload::Any => DamlType::Any,
            DamlTypePayload::TypeRep => DamlType::TypeRep,
            DamlTypePayload::Nat(n) => DamlType::Nat(*n),
        })
    }
}

/// Convert from `DamlTypeVarWrapper` to `DamlTypeVar`.
impl<'a> TryFrom<DamlTypeVarWrapper<'a>> for DamlTypeVar<'a> {
    type Error = DamlLfConvertError;

    fn try_from(typevar: DamlTypeVarWrapper<'a>) -> DamlLfConvertResult<Self> {
        Ok(DamlTypeVar::new(
            typevar.payload.var.resolve(typevar.context.package)?,
            DamlKind::from(&typevar.payload.kind),
        ))
    }
}

/// Convert from `DamlVarWrapper` to `DamlVar`.
impl<'a> TryFrom<&DamlVarWrapper<'a>> for DamlVar<'a> {
    type Error = DamlLfConvertError;

    fn try_from(var: &DamlVarWrapper<'a>) -> Result<Self, Self::Error> {
        let type_arguments = var
            .payload
            .type_arguments
            .iter()
            .map(|ty| DamlType::try_from(&var.wrap(ty)))
            .collect::<DamlLfConvertResult<_>>()?;
        Ok(DamlVar::new(var.payload.var.resolve(var.context.package)?, type_arguments))
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

fn enriched_data(context: DamlPayloadDataWrapper<'_>) -> DamlDataWrapper<'_> {
    DamlDataWrapper::with_data(context, DamlDataEnrichedPayload::from_data_wrapper(context))
}

fn make_data_ref<'a>(
    data_ref: DamlDataRefWrapper<'a>,
    target_data: DamlDataWrapper<'a>,
) -> DamlLfConvertResult<DamlDataRef<'a>> {
    let resolver = data_ref.context.package;
    let current_package_name = data_ref.context.package.name.as_str();
    let target_package_name = target_data.context.package.name.as_str();
    let current_module_path = data_ref.context.module.path.resolve(resolver)?;
    let target_module_path = data_ref.payload.module_path.resolve(resolver)?;
    let data_name = data_ref.payload.data_name.resolve_last(data_ref.context.package)?;
    let type_arguments: Vec<_> = data_ref
        .payload
        .type_arguments
        .iter()
        .map(|ty| DamlType::try_from(&data_ref.wrap(ty)))
        .collect::<DamlLfConvertResult<_>>()?;
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
