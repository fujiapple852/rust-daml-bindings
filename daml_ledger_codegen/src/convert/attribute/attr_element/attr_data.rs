use crate::convert::attribute::attr_element::{extract_enum_data, extract_struct_data, AttrField, AttrType};
use syn::{DataEnum, FieldsNamed};

pub struct AttrRecord {
    pub name: String,
    pub fields: Vec<AttrField>,
}

impl AttrRecord {
    pub fn new(name: String, fields: Vec<AttrField>) -> Self {
        Self {
            name,
            fields,
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
}

impl AttrVariant {
    pub fn new(name: String, fields: Vec<AttrField>) -> Self {
        Self {
            name,
            fields,
        }
    }
}

pub struct AttrEnum {
    pub name: String,
    pub fields: Vec<String>,
}

impl AttrEnum {
    pub fn new(name: String, fields: Vec<String>) -> Self {
        Self {
            name,
            fields,
        }
    }
}

pub fn extract_record(name: String, fields_named: &FieldsNamed) -> AttrRecord {
    let fields: Vec<AttrField> = extract_struct_data(fields_named);
    AttrRecord::new(name, fields)
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

pub fn extract_variant(name: String, data_enum: &DataEnum) -> AttrVariant {
    AttrVariant::new(name, extract_enum_data(data_enum))
}

pub fn extract_enum(name: String, data_enum: &DataEnum) -> AttrEnum {
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
    AttrEnum::new(name, all_constructors)
}
