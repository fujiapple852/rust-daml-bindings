use syn::{DataEnum, Fields, FieldsUnnamed, Variant};

use crate::convert::{AttrField, AttrType};

pub fn extract_enum_data(data_enum: &DataEnum) -> Vec<AttrField> {
    data_enum
        .variants
        .iter()
        .map(|variant| AttrField::new(variant.ident.to_string(), extract_enum_field_type(variant)))
        .collect()
}

fn extract_enum_field_type(variant: &Variant) -> AttrType {
    match &variant.fields {
        Fields::Unit => AttrType::Unit,
        Fields::Unnamed(FieldsUnnamed {
            unnamed,
            ..
        }) =>
            if unnamed.is_empty() {
                AttrType::Unit
            } else if unnamed.len() > 1 {
                panic!(
                    "expected either zero or one type parameter for variant {}, found {}",
                    &variant.ident,
                    unnamed.len()
                )
            } else {
                unnamed.first().map(|pair| AttrType::from_type(&pair.ty)).expect("Field.ty")
            },
        Fields::Named(_) => panic!("only Unnamed or Unit enum variant expected"),
    }
}
