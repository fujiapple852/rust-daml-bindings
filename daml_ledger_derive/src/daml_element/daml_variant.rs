use crate::daml_element::DamlType;
use syn::{DataEnum, Fields, FieldsUnnamed, Variant};

#[derive(Debug)]
pub struct DamlVariant {
    pub variant_name: String,
    pub variant_type: Option<DamlType>,
}

impl DamlVariant {
    pub fn new(variant_name: impl Into<String>, variant_type: impl Into<Option<DamlType>>) -> Self {
        Self {
            variant_name: variant_name.into(),
            variant_type: variant_type.into(),
        }
    }
}

pub fn extract_variant_data(data_enum: &DataEnum) -> Vec<DamlVariant> {
    data_enum
        .variants
        .iter()
        .map(|variant| DamlVariant::new(variant.ident.to_string(), extract_enum_field_type(variant)))
        .collect()
}

fn extract_enum_field_type(variant: &Variant) -> Option<DamlType> {
    match &variant.fields {
        Fields::Unit => None,
        Fields::Unnamed(FieldsUnnamed {
            unnamed,
            ..
        }) =>
            if unnamed.is_empty() {
                None
            } else if unnamed.len() > 1 {
                panic!(format!(
                    "expected either zero or one type parameter for variant {}, found {}",
                    &variant.ident,
                    unnamed.len()
                ))
            } else {
                unnamed.first().map(|pair| DamlType::from_type(&pair.value().ty))
            },
        _ => panic!("only Unnamed or Unit enum variant expected"),
    }
}
