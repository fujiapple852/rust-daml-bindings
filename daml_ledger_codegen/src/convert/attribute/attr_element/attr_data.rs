use crate::convert::attribute::attr_element::{extract_enum_data, extract_struct_data, AttrField, AttrType};
use syn::{DataEnum, FieldsNamed, GenericParam, Generics};

pub struct AttrRecord {
    pub name: String,
    pub fields: Vec<AttrField>,
    pub type_arguments: Vec<String>,
}

impl AttrRecord {
    pub fn new(name: String, fields: Vec<AttrField>, type_arguments: Vec<String>) -> Self {
        Self {
            name,
            fields,
            type_arguments,
        }
    }
}

pub struct AttrTemplate {
    pub name: String,
    pub package_id: String,
    pub module_path: Vec<String>,
    pub fields: Vec<AttrField>,
}

impl AttrTemplate {
    pub fn new(name: String, package_id: String, module_path: Vec<String>, fields: Vec<AttrField>) -> Self {
        Self {
            name,
            package_id,
            module_path,
            fields,
        }
    }
}

pub struct AttrVariant {
    pub name: String,
    pub fields: Vec<AttrField>,
    pub type_arguments: Vec<String>,
}

impl AttrVariant {
    pub fn new(name: String, fields: Vec<AttrField>, type_arguments: Vec<String>) -> Self {
        Self {
            name,
            fields,
            type_arguments,
        }
    }
}

pub struct AttrEnum {
    pub name: String,
    pub fields: Vec<String>,
    pub type_arguments: Vec<String>,
}

impl AttrEnum {
    pub fn new(name: String, fields: Vec<String>, type_arguments: Vec<String>) -> Self {
        Self {
            name,
            fields,
            type_arguments,
        }
    }
}

pub fn extract_record(name: String, fields_named: &FieldsNamed, generics: &Generics) -> AttrRecord {
    let fields: Vec<AttrField> = extract_struct_data(fields_named);
    let type_arguments = extract_generic_type_arguments(generics);
    AttrRecord::new(name, fields, type_arguments)
}

pub fn extract_template(
    name: String,
    package_id: String,
    module_path: String,
    fields_named: &FieldsNamed,
) -> AttrTemplate {
    let module_path: Vec<String> = module_path.split('.').map(ToOwned::to_owned).collect();
    AttrTemplate::new(name, package_id, module_path, extract_struct_data(fields_named))
}

pub fn extract_variant(name: String, data_enum: &DataEnum, generics: &Generics) -> AttrVariant {
    let type_arguments = extract_generic_type_arguments(generics);
    AttrVariant::new(name, extract_enum_data(data_enum), type_arguments)
}

pub fn extract_enum(name: String, data_enum: &DataEnum, generics: &Generics) -> AttrEnum {
    let variants = extract_enum_data(data_enum);
    let all_constructors: Vec<String> = variants
        .into_iter()
        .map(|v| {
            if let AttrType::Unit = v.field_type {
                v.field_label
            } else {
                panic!("DamlEnum variants may not have type parameters (use DamlVariant instead))")
            }
        })
        .collect();
    let type_arguments = extract_generic_type_arguments(generics);
    AttrEnum::new(name, all_constructors, type_arguments)
}

fn extract_generic_type_arguments(generics: &Generics) -> Vec<String> {
    generics
        .params
        .iter()
        .filter_map(|param| {
            if let GenericParam::Type(ty) = param {
                Some(ty.ident.to_string())
            } else {
                None
            }
        })
        .collect()
}
