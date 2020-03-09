use crate::convert::archive::wrapper::payload::type_payload::DamlTypePayload;
use crate::convert::archive::wrapper::payload::util::Required;
use crate::convert::archive::wrapper::payload::InternableString;
use crate::convert::error::{DamlCodeGenError, DamlCodeGenResult};
use daml_lf::protobuf_autogen::daml_lf_1::FieldWithType;
use std::convert::TryFrom;

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

impl<'a> TryFrom<&'a FieldWithType> for DamlFieldPayload<'a> {
    type Error = DamlCodeGenError;

    fn try_from(field_with_type: &'a FieldWithType) -> DamlCodeGenResult<Self> {
        Ok(Self::new(
            InternableString::from(field_with_type.field.as_ref().req()?),
            DamlTypePayload::try_from(field_with_type.r#type.as_ref().req()?)?,
        ))
    }
}
