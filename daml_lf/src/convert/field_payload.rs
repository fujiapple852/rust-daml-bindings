use crate::convert::interned::InternableString;
use crate::convert::type_payload::DamlTypePayload;
use crate::convert::util::Required;
use crate::convert::wrapper::PayloadElementWrapper;
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::FieldWithType;
use std::convert::TryFrom;

///
pub type DamlFieldWrapper<'a> = PayloadElementWrapper<'a, &'a DamlFieldPayload<'a>>;

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
    type Error = DamlLfConvertError;

    fn try_from(field_with_type: &'a FieldWithType) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            InternableString::from(field_with_type.field.as_ref().req()?),
            DamlTypePayload::try_from(field_with_type.r#type.as_ref().req()?)?,
        ))
    }
}
