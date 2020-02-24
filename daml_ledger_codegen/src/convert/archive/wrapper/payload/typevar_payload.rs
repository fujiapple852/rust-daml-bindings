use crate::convert::archive::wrapper::payload::InternableString;
use daml_lf::protobuf_autogen::daml_lf_1::kind::{Arrow, Sum};
use daml_lf::protobuf_autogen::daml_lf_1::{Kind, TypeVarWithKind};

#[derive(Debug)]
pub struct DamlTypeVarPayload<'a> {
    pub var: InternableString<'a>,
    pub kind: DamlKindPayload,
}

impl<'a> From<&'a TypeVarWithKind> for DamlTypeVarPayload<'a> {
    fn from(typevar: &'a TypeVarWithKind) -> Self {
        Self {
            var: InternableString::from(typevar.var.as_ref().expect("TypeVarWithKind.var")),
            kind: DamlKindPayload::from(typevar.kind.as_ref().expect("TypeVarWithKind.kind")),
        }
    }
}

#[derive(Debug)]
pub enum DamlKindPayload {
    Star,
    Arrow(Box<DamlArrowPayload>),
    Nat,
}

impl From<&Kind> for DamlKindPayload {
    fn from(kind: &Kind) -> Self {
        match kind.sum.as_ref().expect("Kind.sum") {
            Sum::Star(_) => DamlKindPayload::Star,
            Sum::Arrow(arrow) => DamlKindPayload::Arrow(Box::new(DamlArrowPayload::from(arrow.as_ref()))),
            Sum::Nat(_) => DamlKindPayload::Nat,
        }
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

impl From<&Arrow> for DamlArrowPayload {
    fn from(arrow: &Arrow) -> Self {
        let params = arrow.params.iter().map(DamlKindPayload::from).collect();
        let result = DamlKindPayload::from(arrow.result.as_ref().expect("Arrow.result").as_ref());
        Self::new(params, result)
    }
}
