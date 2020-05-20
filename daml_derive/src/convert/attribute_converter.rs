use crate::convert::{AttrChoice, AttrEnum, AttrField, AttrRecord, AttrTemplate, AttrType, AttrVariant};
use daml_lf::element::{
    DamlAbsoluteTyCon, DamlChoice, DamlEnum, DamlField, DamlKind, DamlLocalTyCon, DamlRecord, DamlTemplate, DamlTyCon,
    DamlTyConName, DamlType, DamlTypeVarWithKind, DamlVariant,
};

impl<'a> From<&'a AttrRecord> for DamlRecord<'a> {
    fn from(attr_record: &'a AttrRecord) -> Self {
        let fields: Vec<_> = attr_record.fields.iter().map(DamlField::from).collect();
        let type_arguments: Vec<_> =
            attr_record.type_arguments.iter().map(|arg| DamlTypeVarWithKind::new(arg, DamlKind::Star)).collect();
        DamlRecord::new(&attr_record.name, fields, type_arguments, true)
    }
}

impl<'a> From<&'a AttrTemplate> for DamlTemplate<'a> {
    fn from(attr_template: &'a AttrTemplate) -> Self {
        let fields: Vec<DamlField<'_>> = attr_template.fields.iter().map(DamlField::from).collect();
        DamlTemplate::new_with_defaults(
            &attr_template.name,
            &attr_template.package_id,
            to_vec_str(&attr_template.module_path),
            fields,
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
            attr_variant.type_arguments.iter().map(|arg| DamlTypeVarWithKind::new(arg, DamlKind::Star)).collect();
        DamlVariant::new(&attr_variant.name, variant_fields, type_arguments, true)
    }
}

impl<'a> From<&'a AttrEnum> for DamlEnum<'a> {
    fn from(attr_enum: &'a AttrEnum) -> Self {
        let type_arguments: Vec<_> =
            attr_enum.type_arguments.iter().map(|arg| DamlTypeVarWithKind::new(arg, DamlKind::Star)).collect();
        DamlEnum::new(&attr_enum.name, to_vec_str(&attr_enum.fields), type_arguments, true)
    }
}

impl<'a> From<&'a AttrField> for DamlField<'a> {
    fn from(attr_field: &'a AttrField) -> Self {
        DamlField::new(&attr_field.field_label, DamlType::from(&attr_field.field_type))
    }
}

/// The default nat value for `AttrType::Numeric`
pub const DEFAULT_NAT: u8 = 10;

#[allow(clippy::fallible_impl_from)]
impl<'a> From<&'a AttrType> for DamlType<'a> {
    fn from(attr_type: &'a AttrType) -> Self {
        match attr_type {
            AttrType::Unit => DamlType::Unit,
            AttrType::ContractId(_) => DamlType::ContractId(None),
            AttrType::Int64 => DamlType::Int64,
            AttrType::Numeric => DamlType::Numeric(Box::new(DamlType::Nat(DEFAULT_NAT))),
            AttrType::Text => DamlType::Text,
            AttrType::Timestamp => DamlType::Timestamp,
            AttrType::Party => DamlType::Party,
            AttrType::Bool => DamlType::Bool,
            AttrType::Date => DamlType::Date,
            AttrType::List(nested) => DamlType::List(vec![DamlType::from(nested.as_ref())]),
            AttrType::TextMap(nested) => DamlType::TextMap(vec![DamlType::from(nested.as_ref())]),
            AttrType::Optional(nested) => DamlType::Optional(vec![DamlType::from(nested.as_ref())]),
            AttrType::TyCon(data_name, path, type_arguments) =>
                if path.is_empty() {
                    DamlType::TyCon(DamlTyCon::new(
                        DamlTyConName::Local(DamlLocalTyCon::new(data_name, "", to_vec_str(path))),
                        type_arguments.iter().map(DamlType::from).collect(),
                    ))
                } else {
                    DamlType::TyCon(DamlTyCon::new(
                        DamlTyConName::Absolute(DamlAbsoluteTyCon::new(data_name, "", to_vec_str(path))),
                        type_arguments.iter().map(DamlType::from).collect(),
                    ))
                },
            AttrType::Box(boxed_data) => {
                let inner_type = DamlType::from(boxed_data.as_ref());
                match inner_type {
                    DamlType::TyCon(tycon) => DamlType::BoxedTyCon(tycon),
                    _ => panic!("expected TyCon"),
                }
            },
        }
    }
}

fn to_vec_str(input: &[String]) -> Vec<&str> {
    input.iter().map(AsRef::as_ref).collect::<Vec<_>>()
}
