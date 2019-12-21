use crate::convert::archive::wrapper::DamlTypePayload;
use crate::convert::archive::wrapper::PackageInternedResolver;
use crate::convert::archive::wrapper::*;
use crate::element::*;
use daml_lf::LanguageFeatureVersion;
use std::collections::HashMap;

/// Convert from `DamlArchiveWrapper` to `DamlArchive`.
impl<'a> From<&DamlArchiveWrapper<'a>> for DamlArchive<'a> {
    fn from(archive: &DamlArchiveWrapper<'a>) -> Self {
        let packages: HashMap<_, _> = archive.packages().map(|v| (v.payload.name, DamlPackage::from(&v))).collect();
        DamlArchive::new(archive.payload.name, packages)
    }
}

/// Convert from `DamlPackageWrapper` to `DamlPackage`.
impl<'a> From<&DamlPackageWrapper<'a>> for DamlPackage<'a> {
    fn from(package: &DamlPackageWrapper<'a>) -> Self {
        fn from_modules<'a, T: Iterator<Item = DamlModuleWrapper<'a>>>(modules: T) -> DamlModule<'a> {
            modules.fold(DamlModule::new_root(), |mut root, module| {
                let path = module.payload.path.resolve(module.parent_package);
                let data_types: Vec<_> = module.data_types().map(|dt| DamlData::from(&dt)).collect();
                add_module_to_tree(&mut root, data_types, path.clone(), path);
                root
            })
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
        DamlPackage::new(package.payload.name, &package.payload.package_id, from_modules(package.modules()))
    }
}

/// Convert from `DamlDataWrapper` to `DamlData`.
impl<'a> From<&DamlDataWrapper<'a>> for DamlData<'a> {
    fn from(data: &DamlDataWrapper<'a>) -> Self {
        match data {
            DamlDataWrapper::Record(record) => {
                let name = record.payload.name.resolve_last(record.parent_package);
                let fields: Vec<_> = record.fields().map(|f| DamlField::from(&f)).collect();
                DamlData::Record(DamlRecord::new(name, fields))
            },
            DamlDataWrapper::Template(template) => {
                let resolver = template.parent_package;
                let name = template.payload.name.resolve_last(template.parent_package);
                let module_path = template.parent_module.path.resolve(resolver);
                let fields: Vec<_> = template.fields().map(|f| DamlField::from(&f)).collect();
                let choices: Vec<_> = template.choices().map(|c| DamlChoice::from(&c)).collect();
                DamlData::Template(DamlTemplate::new(
                    name,
                    &template.parent_package.package_id,
                    module_path,
                    fields,
                    choices,
                ))
            },
            DamlDataWrapper::Variant(variant) => {
                let name = variant.payload.name.resolve_last(variant.parent_package);
                let fields: Vec<_> = variant.fields().map(|f| DamlField::from(&f)).collect();
                DamlData::Variant(DamlVariant::new(name, fields))
            },
            DamlDataWrapper::Enum(data_enum) => {
                let resolver = data_enum.parent_package;
                let name = data_enum.payload.name.resolve_last(resolver);
                let constructors: Vec<&str> = if data_enum
                    .parent_package
                    .language_version
                    .supports_feature(&LanguageFeatureVersion::INTERNED_STRINGS)
                {
                    assert!(data_enum.payload.constructors_str.is_empty(), "constructors_str should be empty!");
                    resolver.resolve_strings(data_enum.payload.constructors_interned_str)
                } else {
                    assert!(
                        data_enum.payload.constructors_interned_str.is_empty(),
                        "constructors_interned_str should be empty!"
                    );
                    data_enum.payload.constructors_str.iter().map(AsRef::as_ref).collect()
                };
                DamlData::Enum(DamlEnum::new(name, constructors))
            },
        }
    }
}

/// Convert from `DamlChoiceWrapper` to `DamlChoice`.
impl<'a> From<&DamlChoiceWrapper<'a>> for DamlChoice<'a> {
    fn from(choice: &DamlChoiceWrapper<'a>) -> Self {
        let name = choice.payload.name.resolve(choice.parent_package);
        let target_data_type = choice.argument_type().data_ref().get_data();
        match target_data_type {
            DamlDataWrapper::Record(record) => {
                let fields: Vec<_> = record.fields().map(|f| DamlField::from(&f)).collect();
                DamlChoice::new(name, fields, DamlType::from(&choice.return_type()))
            },
            _ => panic!("DAML choice argument must be a Record"),
        }
    }
}

/// Convert from `DamlFieldWrapper` to `DamlField`.
impl<'a> From<&DamlFieldWrapper<'a>> for DamlField<'a> {
    fn from(field: &DamlFieldWrapper<'a>) -> Self {
        DamlField::new(field.payload.name.resolve(field.parent_package), DamlType::from(&field.ty()))
    }
}

/// Convert from `DamlTypeWrapper` to `DamlType`.
impl<'a> From<&DamlTypeWrapper<'a>> for DamlType<'a> {
    fn from(daml_type: &DamlTypeWrapper<'a>) -> Self {
        fn make_data_ref<'a>(
            daml_type: &DamlTypeWrapper<'a>,
            data_ref: DamlDataRefWrapper<'a>,
            target_data: DamlDataWrapper<'a>,
        ) -> DamlDataRef<'a> {
            let resolver = daml_type.parent_package;
            let current_package_name = daml_type.parent_package.name;
            let target_package_name = match target_data {
                DamlDataWrapper::Record(record) => record.parent_package.name,
                DamlDataWrapper::Template(template) => template.parent_package.name,
                DamlDataWrapper::Variant(variant) => variant.parent_package.name,
                DamlDataWrapper::Enum(data_enum) => data_enum.parent_package.name,
            };
            let current_module_path = daml_type.parent_module.path.resolve(resolver);
            let target_module_path = data_ref.payload.module_path.resolve(resolver);
            let data_name = data_ref.payload.data_name.resolve_last(data_ref.parent_package);
            if target_package_name == current_package_name && target_module_path == current_module_path {
                DamlDataRef::Local(DamlLocalDataRef::new(data_name, target_package_name, target_module_path))
            } else {
                DamlDataRef::NonLocal(DamlNonLocalDataRef::new(
                    data_name,
                    current_package_name,
                    current_module_path,
                    target_package_name,
                    target_module_path,
                ))
            }
        }

        match daml_type.payload {
            DamlTypePayload::ContractId(Some(_)) => {
                let data_ref_wrapper = daml_type.contract_id_data_ref();
                let target_data_wrapper = data_ref_wrapper.get_data();
                DamlType::ContractId(Some(make_data_ref(daml_type, data_ref_wrapper, target_data_wrapper)))
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
            DamlTypePayload::List(_) => DamlType::List(Box::new(DamlType::from(&daml_type.nested_type()))),
            DamlTypePayload::Update => DamlType::Update,
            DamlTypePayload::Scenario => DamlType::Scenario,
            DamlTypePayload::TextMap(_) => DamlType::TextMap(Box::new(DamlType::from(&daml_type.nested_type()))),
            DamlTypePayload::Optional(_) => DamlType::Optional(Box::new(DamlType::from(&daml_type.nested_type()))),
            DamlTypePayload::DataRef(_) => {
                let data_ref_wrapper = daml_type.data_ref();
                let target_data_wrapper = data_ref_wrapper.get_data();
                let data_ref = make_data_ref(daml_type, data_ref_wrapper, target_data_wrapper);
                if DamlDataFinder::new(daml_type.parent_data()).find(target_data_wrapper) {
                    DamlType::BoxedDataRef(data_ref)
                } else {
                    DamlType::DataRef(data_ref)
                }
            },
            DamlTypePayload::Var => DamlType::Var,
            DamlTypePayload::Arrow => DamlType::Arrow,
            DamlTypePayload::Any => DamlType::Any,
            DamlTypePayload::TypeRep => DamlType::TypeRep,
        }
    }
}
