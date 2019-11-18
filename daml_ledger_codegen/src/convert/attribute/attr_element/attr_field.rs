use syn::FieldsNamed;

use crate::convert::attribute::attr_element::AttrType;

#[derive(Debug)]
pub struct AttrField {
    pub field_label: String,
    pub field_type: AttrType,
}

impl AttrField {
    pub fn new(field_label: impl Into<String>, field_type: impl Into<AttrType>) -> Self {
        Self {
            field_label: field_label.into(),
            field_type: field_type.into(),
        }
    }
}

pub fn extract_struct_data(fields_named: &FieldsNamed) -> Vec<AttrField> {
    fields_named
        .named
        .iter()
        .map(|field| {
            AttrField::new(
                field.ident.clone().unwrap_or_else(|| unreachable!()).to_string(),
                AttrType::from_type(&field.ty),
            )
        })
        .collect()
}
