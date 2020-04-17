use crate::convert::interned::InternableString;
use crate::convert::util::Required;
use crate::convert::wrapper::PayloadElementWrapper;
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::kind::{Arrow, Sum};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::{Kind, TypeVarWithKind};
use std::convert::TryFrom;

///
pub type DamlTypeVarWrapper<'a> = PayloadElementWrapper<'a, &'a DamlTypeVarPayload<'a>>;

#[derive(Debug)]
pub struct DamlTypeVarPayload<'a> {
    pub var: InternableString<'a>,
    pub kind: DamlKindPayload,
}

impl<'a> TryFrom<&'a TypeVarWithKind> for DamlTypeVarPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(typevar: &'a TypeVarWithKind) -> DamlLfConvertResult<Self> {
        Ok(Self {
            var: InternableString::from(typevar.var.as_ref().req()?),
            kind: DamlKindPayload::try_from(typevar.kind.as_ref().req()?)?,
        })
    }
}

#[derive(Debug)]
pub enum DamlKindPayload {
    Star,
    Arrow(Box<DamlArrowPayload>),
    Nat,
}

impl TryFrom<&Kind> for DamlKindPayload {
    type Error = DamlLfConvertError;

    fn try_from(kind: &Kind) -> DamlLfConvertResult<Self> {
        Ok(match kind.sum.as_ref().req()? {
            Sum::Star(_) => DamlKindPayload::Star,
            Sum::Arrow(arrow) => DamlKindPayload::Arrow(Box::new(DamlArrowPayload::try_from(arrow.as_ref())?)),
            Sum::Nat(_) => DamlKindPayload::Nat,
        })
    }
}

#[derive(Debug)]
pub struct DamlArrowPayload {
    pub params: Vec<DamlKindPayload>,
    pub result: DamlKindPayload,
}

impl DamlArrowPayload {
    pub fn new(params: Vec<DamlKindPayload>, result: DamlKindPayload) -> Self {
        Self {
            params,
            result,
        }
    }
}

impl TryFrom<&Arrow> for DamlArrowPayload {
    type Error = DamlLfConvertError;

    fn try_from(arrow: &Arrow) -> DamlLfConvertResult<Self> {
        let params = arrow.params.iter().map(DamlKindPayload::try_from).collect::<DamlLfConvertResult<_>>()?;
        let result = DamlKindPayload::try_from(arrow.result.as_ref().req()?.as_ref())?;
        Ok(Self::new(params, result))
    }
}
