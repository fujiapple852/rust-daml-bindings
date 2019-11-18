use crate::convert::archive::wrapper::DamlTypePayload;
use crate::convert::archive::wrapper::*;
use crate::element::*;
use std::collections::HashMap;

impl<'a> From<&DamlArchiveWrapper<'a>> for DamlArchive<'a> {
    fn from(archive: &DamlArchiveWrapper<'a>) -> Self {
        let packages: HashMap<_, _> = archive.packages().map(|v| (v.payload.name, DamlPackage::from(&v))).collect();
        DamlArchive::new(archive.payload.name, packages)
    }
}

impl<'a> From<&DamlPackageWrapper<'a>> for DamlPackage<'a> {
    fn from(package: &DamlPackageWrapper<'a>) -> Self {
        DamlPackage::new(&package.payload.name, &package.payload.package_id, DamlModule::from(&package.root_module()))
    }
}

impl<'a> From<&DamlModuleWrapper<'a>> for DamlModule<'a> {
    fn from(module: &DamlModuleWrapper<'a>) -> Self {
        let child_modules: HashMap<_, _> =
            module.child_modules().map(|c| (c.payload.name(), DamlModule::from(&c))).collect();
        let data_types: Vec<_> = module.data_types().map(|dt| DamlData::from(&dt)).collect();
        DamlModule::new(&module.payload.path, child_modules, data_types)
    }
}

impl<'a> From<&DamlDataWrapper<'a>> for DamlData<'a> {
    fn from(data: &DamlDataWrapper<'a>) -> Self {
        match data {
            DamlDataWrapper::Record(record) => {
                let fields: Vec<_> = record.fields().map(|f| DamlField::from(&f)).collect();
                DamlData::Record(DamlRecord::new(record.payload.name, fields))
            },
            DamlDataWrapper::Template(template) => {
                let fields: Vec<_> = template.fields().map(|f| DamlField::from(&f)).collect();
                let choices: Vec<_> = template.choices().map(|c| DamlChoice::from(&c)).collect();
                DamlData::Template(DamlTemplate::new(
                    template.payload.name,
                    &template.parent_package.package_id,
                    &template.parent_module.path,
                    fields,
                    choices,
                ))
            },
            DamlDataWrapper::Variant(variant) => {
                let fields: Vec<_> = variant.fields().map(|f| DamlField::from(&f)).collect();
                DamlData::Variant(DamlVariant::new(variant.payload.name, fields))
            },
            DamlDataWrapper::Enum(data_enum) =>
                DamlData::Enum(DamlEnum::new(data_enum.payload.name, &data_enum.payload.constructors)),
        }
    }
}

impl<'a> From<&DamlChoiceWrapper<'a>> for DamlChoice<'a> {
    fn from(choice: &DamlChoiceWrapper<'a>) -> Self {
        let target_data_type = choice.argument_type().data_ref().get_data();
        match target_data_type {
            DamlDataWrapper::Record(record) => {
                let fields: Vec<_> = record.fields().map(|f| DamlField::from(&f)).collect();
                DamlChoice::new(&choice.payload.name, fields, DamlType::from(&choice.return_type()))
            },
            _ => panic!("DAML choice argument must be a Record"),
        }
    }
}

impl<'a> From<&DamlFieldWrapper<'a>> for DamlField<'a> {
    fn from(field: &DamlFieldWrapper<'a>) -> Self {
        DamlField::new(&field.payload.name, DamlType::from(&field.ty()))
    }
}

impl<'a> From<&DamlTypeWrapper<'a>> for DamlType<'a> {
    fn from(daml_type: &DamlTypeWrapper<'a>) -> Self {
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
        }
    }
}

fn make_data_ref<'a>(
    daml_type: &DamlTypeWrapper<'a>,
    data_ref: DamlDataRefWrapper<'a>,
    target_data: DamlDataWrapper<'a>,
) -> DamlDataRef<'a> {
    let current_module_path = &daml_type.parent_module.path;
    let current_package_name = daml_type.parent_package.name;
    let target_module_path = &data_ref.payload.module_path;
    let target_package_name = match target_data {
        DamlDataWrapper::Record(record) => record.parent_package.name,
        DamlDataWrapper::Template(template) => template.parent_package.name,
        DamlDataWrapper::Variant(variant) => variant.parent_package.name,
        DamlDataWrapper::Enum(data_enum) => data_enum.parent_package.name,
    };
    if target_package_name == current_package_name && target_module_path == current_module_path {
        DamlDataRef::Local(DamlLocalDataRef::new(&data_ref.payload.data_name, target_package_name, target_module_path))
    } else {
        DamlDataRef::NonLocal(DamlNonLocalDataRef::new(
            &data_ref.payload.data_name,
            current_package_name,
            current_module_path,
            target_package_name,
            target_module_path,
        ))
    }
}
