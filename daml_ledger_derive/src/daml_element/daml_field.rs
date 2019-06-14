use crate::daml_element::DamlType;
use syn::FieldsNamed;

#[derive(Debug)]
pub struct DamlField {
    pub field_label: String,
    pub field_type: DamlType,
}

impl DamlField {
    pub fn new(field_label: impl Into<String>, field_type: impl Into<DamlType>) -> Self {
        Self {
            field_label: field_label.into(),
            field_type: field_type.into(),
        }
    }
}

pub fn extract_struct_data(fields_named: &FieldsNamed) -> Vec<DamlField> {
    fields_named
        .named
        .iter()
        .map(|field| {
            DamlField::new(
                field.ident.clone().unwrap_or_else(|| unreachable!()).to_string(),
                DamlType::from_type(&field.ty),
            )
        })
        .collect()
}
