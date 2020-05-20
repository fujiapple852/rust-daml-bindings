use crate::convert::expr_payload::DamlExprPayload;
use crate::convert::interned::InternableDottedName;
use crate::convert::type_payload::DamlTypePayload;
use crate::convert::util::Required;
use crate::convert::wrapper::PayloadElementWrapper;
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::DefValue;
use std::convert::TryFrom;

///
pub type DamlDefValueWrapper<'a> = PayloadElementWrapper<'a, &'a DamlDefValuePayload<'a>>;

#[derive(Debug)]
pub struct DamlDefValuePayload<'a> {
    pub name: InternableDottedName<'a>,
    pub ty: DamlTypePayload<'a>,
    pub expr: DamlExprPayload<'a>,
    pub no_party_literals: bool,
    pub is_test: bool,
}

impl<'a> DamlDefValuePayload<'a> {
    pub const fn new(
        name: InternableDottedName<'a>,
        ty: DamlTypePayload<'a>,
        expr: DamlExprPayload<'a>,
        no_party_literals: bool,
        is_test: bool,
    ) -> Self {
        Self {
            name,
            ty,
            expr,
            no_party_literals,
            is_test,
        }
    }
}

impl<'a> TryFrom<&'a DefValue> for DamlDefValuePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(def_value: &'a DefValue) -> DamlLfConvertResult<Self> {
        let name_with_type = def_value.name_with_type.as_ref().req()?;
        Ok(Self::new(
            InternableDottedName::new_implied(name_with_type.name_interned_dname, name_with_type.name_dname.as_slice()),
            DamlTypePayload::try_from(name_with_type.r#type.as_ref().req()?)?,
            DamlExprPayload::try_from(def_value.expr.as_ref().req()?)?,
            def_value.no_party_literals,
            def_value.is_test,
        ))
    }
}
