use crate::convert::archive::wrapper::payload::type_payload::DamlTypePayload;
use crate::convert::archive::wrapper::payload::InternableString;
use daml_lf::protobuf_autogen::daml_lf_1::FieldWithType;

#[derive(Debug)]
pub struct DamlFieldPayload<'a> {
    pub name: InternableString<'a>,
    pub ty: DamlTypePayload<'a>,
}

impl<'a> DamlFieldPayload<'a> {
    pub fn new(name: InternableString<'a>, ty: DamlTypePayload<'a>) -> Self {
        Self {
            name,
            ty,
        }
    }
}

impl<'a> From<&'a FieldWithType> for DamlFieldPayload<'a> {
    fn from(field_with_type: &'a FieldWithType) -> Self {
        Self::new(
            InternableString::from(field_with_type.field.as_ref().expect("FieldWithType.field")),
            DamlTypePayload::from(field_with_type.r#type.as_ref().expect("FieldWithType.r#type")),
        )
    }
}
