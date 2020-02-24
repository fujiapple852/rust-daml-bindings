use crate::convert::attribute::attr_element::{
    AttrChoice, AttrEnum, AttrField, AttrRecord, AttrTemplate, AttrType, AttrVariant,
};
use crate::element::{
    DamlAbsoluteDataRef, DamlChoice, DamlDataRef, DamlEnum, DamlField, DamlKind, DamlLocalDataRef, DamlRecord,
    DamlTemplate, DamlType, DamlTypeVar, DamlVariant,
};

impl<'a> From<&'a AttrRecord> for DamlRecord<'a> {
    fn from(attr_record: &'a AttrRecord) -> Self {
        let fields: Vec<_> = attr_record.fields.iter().map(DamlField::from).collect();
        let type_arguments: Vec<_> =
            attr_record.type_arguments.iter().map(|arg| DamlTypeVar::new(arg, DamlKind::Star)).collect();
        DamlRecord::new(&attr_record.name, fields, type_arguments)
    }
}

impl<'a> From<&'a AttrTemplate> for DamlTemplate<'a> {
    fn from(attr_template: &'a AttrTemplate) -> Self {
        let fields: Vec<DamlField> = attr_template.fields.iter().map(DamlField::from).collect();
        DamlTemplate::new(
            &attr_template.name,
            &attr_template.package_id,
            to_vec_str(&attr_template.module_path),
            fields,
            vec![],
        )
    }
}

impl<'a> From<&'a AttrChoice> for DamlChoice<'a> {
    fn from(attr_choice: &'a AttrChoice) -> Self {
        DamlChoice::new(
            &attr_choice.choice_name,
            attr_choice.choice_arguments.iter().map(DamlField::from).collect(),
            DamlType::from(&attr_choice.choice_return_type),
        )
    }
}

impl<'a> From<&'a AttrVariant> for DamlVariant<'a> {
    fn from(attr_variant: &'a AttrVariant) -> Self {
        let variant_fields = attr_variant.fields.iter().map(DamlField::from).collect();
        let type_arguments: Vec<_> =
            attr_variant.type_arguments.iter().map(|arg| DamlTypeVar::new(arg, DamlKind::Star)).collect();
        DamlVariant::new(&attr_variant.name, variant_fields, type_arguments)
    }
}

impl<'a> From<&'a AttrEnum> for DamlEnum<'a> {
    fn from(attr_enum: &'a AttrEnum) -> Self {
        let type_arguments: Vec<_> =
            attr_enum.type_arguments.iter().map(|arg| DamlTypeVar::new(arg, DamlKind::Star)).collect();
        DamlEnum::new(&attr_enum.name, to_vec_str(&attr_enum.fields), type_arguments)
    }
}

impl<'a> From<&'a AttrField> for DamlField<'a> {
    fn from(attr_field: &'a AttrField) -> Self {
        DamlField::new(&attr_field.field_label, DamlType::from(&attr_field.field_type))
    }
}

impl<'a> From<&'a AttrType> for DamlType<'a> {
    fn from(attr_type: &'a AttrType) -> Self {
        match attr_type {
            AttrType::Unit => DamlType::Unit,
            AttrType::ContractId(_) => DamlType::ContractId(None),
            AttrType::Int64 => DamlType::Int64,
            AttrType::Numeric => DamlType::Numeric,
            AttrType::Text => DamlType::Text,
            AttrType::Timestamp => DamlType::Timestamp,
            AttrType::Party => DamlType::Party,
            AttrType::Bool => DamlType::Bool,
            AttrType::Date => DamlType::Date,
            AttrType::List(nested) => DamlType::List(Box::new(DamlType::from(nested.as_ref()))),
            AttrType::TextMap(nested) => DamlType::TextMap(Box::new(DamlType::from(nested.as_ref()))),
            AttrType::Optional(nested) => DamlType::Optional(Box::new(DamlType::from(nested.as_ref()))),
            AttrType::DataRef(data_name, path, type_arguments) =>
                if path.is_empty() {
                    DamlType::DataRef(DamlDataRef::Local(DamlLocalDataRef::new(
                        data_name,
                        "",
                        to_vec_str(path),
                        type_arguments.iter().map(DamlType::from).collect(),
                    )))
                } else {
                    DamlType::DataRef(DamlDataRef::Absolute(DamlAbsoluteDataRef::new(
                        data_name,
                        "",
                        to_vec_str(path),
                        type_arguments.iter().map(DamlType::from).collect(),
                    )))
                },
            AttrType::Box(boxed_data) => {
                let inner_type = DamlType::from(boxed_data.as_ref());
                match inner_type {
                    DamlType::DataRef(data_ref) => DamlType::BoxedDataRef(data_ref),
                    _ => panic!("expected DataRef"),
                }
            },
        }
    }
}

fn to_vec_str(input: &[String]) -> Vec<&str> {
    input.iter().map(AsRef::as_ref).collect::<Vec<_>>()
}
