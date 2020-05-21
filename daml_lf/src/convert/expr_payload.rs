use std::convert::TryFrom;

use crate::convert::interned::{InternableDottedName, InternableString};
use crate::convert::type_payload::{DamlPackageRefPayload, DamlTyConNamePayload, DamlTyConPayload, DamlTypePayload};
use crate::convert::typevar_payload::DamlTypeVarWithKindPayload;
use crate::convert::util::Required;
use crate::convert::wrapper::PayloadElementWrapper;
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::case_alt::Sum;
use crate::lf_protobuf::com::digitalasset::daml_lf_1::def_template::def_key;
use crate::lf_protobuf::com::digitalasset::daml_lf_1::expr::{
    Abs, App, Cons, EnumCon, FromAny, FromAnyException, OptionalSome, RecCon, RecProj, RecUpd, StructCon, StructProj,
    StructUpd, Throw, ToAny, ToAnyException, TyAbs, TyApp, VariantCon,
};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::scenario::Commit;
use crate::lf_protobuf::com::digitalasset::daml_lf_1::update;
use crate::lf_protobuf::com::digitalasset::daml_lf_1::update::{
    Create, Exercise, ExerciseByKey, Fetch, RetrieveByKey, TryCatch,
};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::{case_alt, scenario, CaseAlt};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::{
    expr, prim_lit, Binding, Block, BuiltinFunction, Case, Expr, FieldWithExpr, ModuleRef, PrimCon, PrimLit, Pure,
    Scenario, Update, ValName, VarWithType,
};

///
pub type DamlExprWrapper<'a> = PayloadElementWrapper<'a, &'a DamlExprPayload<'a>>;

#[derive(Debug)]
pub enum DamlExprPayload<'a> {
    Var(InternableString<'a>),
    Val(DamlValueNamePayload<'a>),
    Builtin(DamlBuiltinFunctionPayload),
    PrimCon(DamlPrimConPayload),
    PrimLit(DamlPrimLitPayload<'a>),
    RecCon(DamlRecConPayload<'a>),
    RecProj(DamlRecProjPayload<'a>),
    RecUpd(DamlRecUpdPayload<'a>),
    VariantCon(DamlVariantConPayload<'a>),
    EnumCon(DamlEnumConPayload<'a>),
    StructCon(DamlStructConPayload<'a>),
    StructProj(DamlStructProjPayload<'a>),
    StructUpd(DamlStructUpdPayload<'a>),
    App(DamlAppPayload<'a>),
    TyApp(DamlTyAppPayload<'a>),
    Abs(DamlAbsPayload<'a>),
    TyAbs(DamlTyAbsPayload<'a>),
    Case(DamlCasePayload<'a>),
    Let(DamlBlockPayload<'a>),
    Nil(DamlTypePayload<'a>),
    Cons(DamlConsPayload<'a>),
    Update(DamlUpdatePayload<'a>),
    Scenario(DamlScenarioPayload<'a>),
    OptionalNone(DamlTypePayload<'a>),
    OptionalSome(DamlOptionalSomePayload<'a>),
    ToAny(DamlToAnyPayload<'a>),
    FromAny(DamlFromAnyPayload<'a>),
    TypeRep(DamlTypePayload<'a>),
    ToAnyException(DamlToAnyExceptionPayload<'a>),
    FromAnyException(DamlFromAnyExceptionPayload<'a>),
    Throw(DamlThrowPayload<'a>),
}

impl<'a> TryFrom<&'a Expr> for DamlExprPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(expr: &'a Expr) -> DamlLfConvertResult<Self> {
        Ok(match expr.sum.as_ref().req()? {
            expr::Sum::VarStr(var_str) => DamlExprPayload::Var(InternableString::LiteralString(var_str.as_str())),
            expr::Sum::VarInternedStr(i) => DamlExprPayload::Var(InternableString::InternedString(*i)),
            expr::Sum::Val(val_name) => DamlExprPayload::Val(DamlValueNamePayload::try_from(val_name)?),
            expr::Sum::Builtin(i) => DamlExprPayload::Builtin(DamlBuiltinFunctionPayload::try_from(i)?),
            expr::Sum::PrimCon(i) => DamlExprPayload::PrimCon(DamlPrimConPayload::try_from(i)?),
            expr::Sum::PrimLit(prim_lit) => DamlExprPayload::PrimLit(DamlPrimLitPayload::try_from(prim_lit)?),
            expr::Sum::RecCon(rec_con) => DamlExprPayload::RecCon(DamlRecConPayload::try_from(rec_con)?),
            expr::Sum::RecProj(rec_proj) => DamlExprPayload::RecProj(DamlRecProjPayload::try_from(rec_proj.as_ref())?),
            expr::Sum::RecUpd(rec_upd) => DamlExprPayload::RecUpd(DamlRecUpdPayload::try_from(rec_upd.as_ref())?),
            expr::Sum::VariantCon(variant_con) =>
                DamlExprPayload::VariantCon(DamlVariantConPayload::try_from(variant_con.as_ref())?),
            expr::Sum::EnumCon(enum_con) => DamlExprPayload::EnumCon(DamlEnumConPayload::try_from(enum_con)?),
            expr::Sum::StructCon(struct_con) => DamlExprPayload::StructCon(DamlStructConPayload::try_from(struct_con)?),
            expr::Sum::StructProj(struct_proj) =>
                DamlExprPayload::StructProj(DamlStructProjPayload::try_from(struct_proj.as_ref())?),
            expr::Sum::StructUpd(struct_upd) =>
                DamlExprPayload::StructUpd(DamlStructUpdPayload::try_from(struct_upd.as_ref())?),
            expr::Sum::App(app) => DamlExprPayload::App(DamlAppPayload::try_from(app.as_ref())?),
            expr::Sum::TyApp(ty_app) => DamlExprPayload::TyApp(DamlTyAppPayload::try_from(ty_app.as_ref())?),
            expr::Sum::Abs(abs) => DamlExprPayload::Abs(DamlAbsPayload::try_from(abs.as_ref())?),
            expr::Sum::TyAbs(ty_abs) => DamlExprPayload::TyAbs(DamlTyAbsPayload::try_from(ty_abs.as_ref())?),
            expr::Sum::Case(case) => DamlExprPayload::Case(DamlCasePayload::try_from(case.as_ref())?),
            expr::Sum::Let(block) => DamlExprPayload::Let(DamlBlockPayload::try_from(block.as_ref())?),
            expr::Sum::Nil(nil) => DamlExprPayload::Nil(DamlTypePayload::try_from(nil.r#type.as_ref().req()?)?),
            expr::Sum::Cons(cons) => DamlExprPayload::Cons(DamlConsPayload::try_from(cons.as_ref())?),
            expr::Sum::Update(update) => DamlExprPayload::Update(DamlUpdatePayload::try_from(update.as_ref())?),
            expr::Sum::Scenario(scenario) =>
                DamlExprPayload::Scenario(DamlScenarioPayload::try_from(scenario.as_ref())?),
            expr::Sum::OptionalNone(opt_none) =>
                DamlExprPayload::OptionalNone(DamlTypePayload::try_from(opt_none.r#type.as_ref().req()?)?),
            expr::Sum::OptionalSome(opt_some) =>
                DamlExprPayload::OptionalSome(DamlOptionalSomePayload::try_from(opt_some.as_ref())?),
            expr::Sum::ToAny(to_any) => DamlExprPayload::ToAny(DamlToAnyPayload::try_from(to_any.as_ref())?),
            expr::Sum::FromAny(from_any) => DamlExprPayload::FromAny(DamlFromAnyPayload::try_from(from_any.as_ref())?),
            expr::Sum::TypeRep(ty) => DamlExprPayload::TypeRep(DamlTypePayload::try_from(ty)?),
            expr::Sum::ToAnyException(to_any_exception) =>
                DamlExprPayload::ToAnyException(DamlToAnyExceptionPayload::try_from(to_any_exception.as_ref())?),
            expr::Sum::FromAnyException(from_any_exception) =>
                DamlExprPayload::FromAnyException(DamlFromAnyExceptionPayload::try_from(from_any_exception.as_ref())?),
            expr::Sum::Throw(throw) => DamlExprPayload::Throw(DamlThrowPayload::try_from(throw.as_ref())?),
        })
    }
}

pub type DamlCaseWrapper<'a> = PayloadElementWrapper<'a, &'a DamlCasePayload<'a>>;

#[derive(Debug)]
pub struct DamlCasePayload<'a> {
    pub scrut: Box<DamlExprPayload<'a>>,
    pub alts: Vec<DamlCaseAltPayload<'a>>,
}

impl<'a> DamlCasePayload<'a> {
    pub fn new(scrut: Box<DamlExprPayload<'a>>, alts: Vec<DamlCaseAltPayload<'a>>) -> Self {
        Self {
            scrut,
            alts,
        }
    }
}

impl<'a> TryFrom<&'a Case> for DamlCasePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(case: &'a Case) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            Box::new(DamlExprPayload::try_from(case.scrut.as_ref().req()?.as_ref())?),
            case.alts.iter().map(DamlCaseAltPayload::try_from).collect::<DamlLfConvertResult<_>>()?,
        ))
    }
}

pub type DamlCaseAltWrapper<'a> = PayloadElementWrapper<'a, &'a DamlCaseAltPayload<'a>>;

#[derive(Debug)]
pub struct DamlCaseAltPayload<'a> {
    pub body: DamlExprPayload<'a>,
    pub sum: DamlCaseAltSumPayload<'a>,
}

impl<'a> DamlCaseAltPayload<'a> {
    pub fn new(body: DamlExprPayload<'a>, sum: DamlCaseAltSumPayload<'a>) -> Self {
        Self {
            body,
            sum,
        }
    }
}

impl<'a> TryFrom<&'a CaseAlt> for DamlCaseAltPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(case_alt: &'a CaseAlt) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlExprPayload::try_from(case_alt.body.as_ref().req()?)?,
            DamlCaseAltSumPayload::try_from(case_alt.sum.as_ref().req()?)?,
        ))
    }
}

pub type DamlCaseAltSumWrapper<'a> = PayloadElementWrapper<'a, &'a DamlCaseAltSumPayload<'a>>;

#[derive(Debug)]
pub enum DamlCaseAltSumPayload<'a> {
    Default,
    Variant(DamlCaseAltVariantPayload<'a>),
    PrimCon(DamlPrimConPayload),
    Nil,
    Cons(DamlCaseAltConsPayload<'a>),
    OptionalNone,
    OptionalSome(DamlCaseAltOptionalSomePayload<'a>),
    Enum(DamlCaseAltEnumPayload<'a>),
}

impl<'a> TryFrom<&'a case_alt::Sum> for DamlCaseAltSumPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(sum: &'a case_alt::Sum) -> DamlLfConvertResult<Self> {
        Ok(match sum {
            Sum::Default(_) => DamlCaseAltSumPayload::Default,
            Sum::Variant(variant) => DamlCaseAltSumPayload::Variant(DamlCaseAltVariantPayload::try_from(variant)?),
            Sum::PrimCon(prim_con) => DamlCaseAltSumPayload::PrimCon(DamlPrimConPayload::try_from(prim_con)?),
            Sum::Nil(_) => DamlCaseAltSumPayload::Nil,
            Sum::Cons(cons) => DamlCaseAltSumPayload::Cons(DamlCaseAltConsPayload::try_from(cons)?),
            Sum::OptionalNone(_) => DamlCaseAltSumPayload::OptionalNone,
            Sum::OptionalSome(opt_some) =>
                DamlCaseAltSumPayload::OptionalSome(DamlCaseAltOptionalSomePayload::try_from(opt_some)?),
            Sum::Enum(enum_alt) => DamlCaseAltSumPayload::Enum(DamlCaseAltEnumPayload::try_from(enum_alt)?),
        })
    }
}

pub type DamlCaseAltVariantWrapper<'a> = PayloadElementWrapper<'a, &'a DamlCaseAltVariantPayload<'a>>;

#[derive(Debug)]
pub struct DamlCaseAltVariantPayload<'a> {
    pub con: DamlTyConNamePayload<'a>,
    pub variant: InternableString<'a>,
    pub binder: InternableString<'a>,
}

impl<'a> DamlCaseAltVariantPayload<'a> {
    pub fn new(con: DamlTyConNamePayload<'a>, variant: InternableString<'a>, binder: InternableString<'a>) -> Self {
        Self {
            con,
            variant,
            binder,
        }
    }
}

impl<'a> TryFrom<&'a case_alt::Variant> for DamlCaseAltVariantPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(variant_alt: &'a case_alt::Variant) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTyConNamePayload::try_from(variant_alt.con.as_ref().req()?)?,
            InternableString::from(variant_alt.variant.as_ref().req()?),
            InternableString::from(variant_alt.binder.as_ref().req()?),
        ))
    }
}

pub type DamlCaseAltEnumWrapper<'a> = PayloadElementWrapper<'a, &'a DamlCaseAltEnumPayload<'a>>;

#[derive(Debug)]
pub struct DamlCaseAltEnumPayload<'a> {
    pub con: DamlTyConNamePayload<'a>,
    pub constructor: InternableString<'a>,
}

impl<'a> DamlCaseAltEnumPayload<'a> {
    pub fn new(con: DamlTyConNamePayload<'a>, constructor: InternableString<'a>) -> Self {
        Self {
            con,
            constructor,
        }
    }
}

impl<'a> TryFrom<&'a case_alt::Enum> for DamlCaseAltEnumPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(enum_alt: &'a case_alt::Enum) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTyConNamePayload::try_from(enum_alt.con.as_ref().req()?)?,
            InternableString::from(enum_alt.constructor.as_ref().req()?),
        ))
    }
}

pub type DamlCaseAltConsWrapper<'a> = PayloadElementWrapper<'a, &'a DamlCaseAltConsPayload<'a>>;

#[derive(Debug)]
pub struct DamlCaseAltConsPayload<'a> {
    pub var_head: InternableString<'a>,
    pub var_tail: InternableString<'a>,
}

impl<'a> DamlCaseAltConsPayload<'a> {
    pub fn new(var_head: InternableString<'a>, var_tail: InternableString<'a>) -> Self {
        Self {
            var_head,
            var_tail,
        }
    }
}

impl<'a> TryFrom<&'a case_alt::Cons> for DamlCaseAltConsPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(cons_alt: &'a case_alt::Cons) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            InternableString::from(cons_alt.var_head.as_ref().req()?),
            InternableString::from(cons_alt.var_tail.as_ref().req()?),
        ))
    }
}

pub type DamlCaseAltOptionalSomeWrapper<'a> = PayloadElementWrapper<'a, &'a DamlCaseAltOptionalSomePayload<'a>>;

#[derive(Debug)]
pub struct DamlCaseAltOptionalSomePayload<'a> {
    pub var_body: InternableString<'a>,
}

impl<'a> DamlCaseAltOptionalSomePayload<'a> {
    pub fn new(var_body: InternableString<'a>) -> Self {
        Self {
            var_body,
        }
    }
}

impl<'a> TryFrom<&'a case_alt::OptionalSome> for DamlCaseAltOptionalSomePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(opt_some_alt: &'a case_alt::OptionalSome) -> DamlLfConvertResult<Self> {
        Ok(Self::new(InternableString::from(opt_some_alt.var_body.as_ref().req()?)))
    }
}

pub type DamlOptionalSomeWrapper<'a> = PayloadElementWrapper<'a, &'a DamlOptionalSomePayload<'a>>;

#[derive(Debug)]
pub struct DamlOptionalSomePayload<'a> {
    pub ty: DamlTypePayload<'a>,
    pub body: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlOptionalSomePayload<'a> {
    pub fn new(ty: DamlTypePayload<'a>, body: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            ty,
            body,
        }
    }
}

impl<'a> TryFrom<&'a OptionalSome> for DamlOptionalSomePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(opt_some: &'a OptionalSome) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTypePayload::try_from(opt_some.r#type.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(opt_some.body.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlToAnyWrapper<'a> = PayloadElementWrapper<'a, &'a DamlToAnyPayload<'a>>;

#[derive(Debug)]
pub struct DamlToAnyPayload<'a> {
    pub ty: DamlTypePayload<'a>,
    pub expr: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlToAnyPayload<'a> {
    pub fn new(ty: DamlTypePayload<'a>, expr: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            ty,
            expr,
        }
    }
}

impl<'a> TryFrom<&'a ToAny> for DamlToAnyPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(to_any: &'a ToAny) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTypePayload::try_from(to_any.r#type.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(to_any.expr.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlFromAnyWrapper<'a> = PayloadElementWrapper<'a, &'a DamlFromAnyPayload<'a>>;

#[derive(Debug)]
pub struct DamlFromAnyPayload<'a> {
    pub ty: DamlTypePayload<'a>,
    pub expr: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlFromAnyPayload<'a> {
    pub fn new(ty: DamlTypePayload<'a>, expr: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            ty,
            expr,
        }
    }
}

impl<'a> TryFrom<&'a FromAny> for DamlFromAnyPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(from_any: &'a FromAny) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTypePayload::try_from(from_any.r#type.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(from_any.expr.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlBlockWrapper<'a> = PayloadElementWrapper<'a, &'a DamlBlockPayload<'a>>;

#[derive(Debug)]
pub struct DamlBlockPayload<'a> {
    pub bindings: Vec<DamlBindingPayload<'a>>,
    pub body: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlBlockPayload<'a> {
    pub fn new(bindings: Vec<DamlBindingPayload<'a>>, body: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            bindings,
            body,
        }
    }
}

impl<'a> TryFrom<&'a Block> for DamlBlockPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(block: &'a Block) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            block.bindings.iter().map(DamlBindingPayload::try_from).collect::<DamlLfConvertResult<_>>()?,
            Box::new(DamlExprPayload::try_from(block.body.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlBindingWrapper<'a> = PayloadElementWrapper<'a, &'a DamlBindingPayload<'a>>;

#[derive(Debug)]
pub struct DamlBindingPayload<'a> {
    pub binder: DamlVarWithTypePayload<'a>,
    pub bound: DamlExprPayload<'a>,
}

impl<'a> DamlBindingPayload<'a> {
    pub fn new(binder: DamlVarWithTypePayload<'a>, bound: DamlExprPayload<'a>) -> Self {
        Self {
            binder,
            bound,
        }
    }
}

impl<'a> TryFrom<&'a Binding> for DamlBindingPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(binding: &'a Binding) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlVarWithTypePayload::try_from(binding.binder.as_ref().req()?)?,
            DamlExprPayload::try_from(binding.bound.as_ref().req()?)?,
        ))
    }
}

pub type DamlTyAbsWrapper<'a> = PayloadElementWrapper<'a, &'a DamlTyAbsPayload<'a>>;

#[derive(Debug)]
pub struct DamlTyAbsPayload<'a> {
    pub params: Vec<DamlTypeVarWithKindPayload<'a>>,
    pub body: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlTyAbsPayload<'a> {
    pub fn new(params: Vec<DamlTypeVarWithKindPayload<'a>>, body: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            params,
            body,
        }
    }
}

impl<'a> TryFrom<&'a TyAbs> for DamlTyAbsPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(ty_abs: &'a TyAbs) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            ty_abs.param.iter().map(DamlTypeVarWithKindPayload::try_from).collect::<DamlLfConvertResult<_>>()?,
            Box::new(DamlExprPayload::try_from(ty_abs.body.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlAbsWrapper<'a> = PayloadElementWrapper<'a, &'a DamlAbsPayload<'a>>;

#[derive(Debug)]
pub struct DamlAbsPayload<'a> {
    pub params: Vec<DamlVarWithTypePayload<'a>>,
    pub body: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlAbsPayload<'a> {
    pub fn new(params: Vec<DamlVarWithTypePayload<'a>>, body: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            params,
            body,
        }
    }
}

impl<'a> TryFrom<&'a Abs> for DamlAbsPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(abs: &'a Abs) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            abs.param.iter().map(DamlVarWithTypePayload::try_from).collect::<DamlLfConvertResult<_>>()?,
            Box::new(DamlExprPayload::try_from(abs.body.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlVarWithTypeWrapper<'a> = PayloadElementWrapper<'a, &'a DamlVarWithTypePayload<'a>>;

#[derive(Debug)]
pub struct DamlVarWithTypePayload<'a> {
    pub ty: DamlTypePayload<'a>,
    pub var: InternableString<'a>,
}

impl<'a> DamlVarWithTypePayload<'a> {
    pub fn new(ty: DamlTypePayload<'a>, var: InternableString<'a>) -> Self {
        Self {
            ty,
            var,
        }
    }
}

impl<'a> TryFrom<&'a VarWithType> for DamlVarWithTypePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(var_with_type: &'a VarWithType) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTypePayload::try_from(var_with_type.r#type.as_ref().req()?)?,
            InternableString::from(var_with_type.var.as_ref().req()?),
        ))
    }
}

pub type DamlTyAppWrapper<'a> = PayloadElementWrapper<'a, &'a DamlTyAppPayload<'a>>;

#[derive(Debug)]
pub struct DamlTyAppPayload<'a> {
    pub expr: Box<DamlExprPayload<'a>>,
    pub types: Vec<DamlTypePayload<'a>>,
}

impl<'a> DamlTyAppPayload<'a> {
    pub fn new(expr: Box<DamlExprPayload<'a>>, types: Vec<DamlTypePayload<'a>>) -> Self {
        Self {
            expr,
            types,
        }
    }
}

impl<'a> TryFrom<&'a TyApp> for DamlTyAppPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(ty_app: &'a TyApp) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            Box::new(DamlExprPayload::try_from(ty_app.expr.as_ref().req()?.as_ref())?),
            ty_app.types.iter().map(DamlTypePayload::try_from).collect::<DamlLfConvertResult<_>>()?,
        ))
    }
}

pub type DamlAppWrapper<'a> = PayloadElementWrapper<'a, &'a DamlAppPayload<'a>>;

#[derive(Debug)]
pub struct DamlAppPayload<'a> {
    pub fun: Box<DamlExprPayload<'a>>,
    pub args: Vec<DamlExprPayload<'a>>,
}

impl<'a> DamlAppPayload<'a> {
    pub fn new(fun: Box<DamlExprPayload<'a>>, args: Vec<DamlExprPayload<'a>>) -> Self {
        Self {
            fun,
            args,
        }
    }
}

impl<'a> TryFrom<&'a App> for DamlAppPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(app: &'a App) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            Box::new(DamlExprPayload::try_from(app.fun.as_ref().req()?.as_ref())?),
            app.args.iter().map(DamlExprPayload::try_from).collect::<DamlLfConvertResult<_>>()?,
        ))
    }
}

pub type DamlStructUpdWrapper<'a> = PayloadElementWrapper<'a, &'a DamlStructUpdPayload<'a>>;

#[derive(Debug)]
pub struct DamlStructUpdPayload<'a> {
    pub struct_expr: Box<DamlExprPayload<'a>>,
    pub update: Box<DamlExprPayload<'a>>,
    pub field: InternableString<'a>,
}

impl<'a> DamlStructUpdPayload<'a> {
    pub fn new(
        struct_expr: Box<DamlExprPayload<'a>>,
        update: Box<DamlExprPayload<'a>>,
        field: InternableString<'a>,
    ) -> Self {
        Self {
            struct_expr,
            update,
            field,
        }
    }
}

impl<'a> TryFrom<&'a StructUpd> for DamlStructUpdPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(struct_upd: &'a StructUpd) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            Box::new(DamlExprPayload::try_from(struct_upd.r#struct.as_ref().req()?.as_ref())?),
            Box::new(DamlExprPayload::try_from(struct_upd.update.as_ref().req()?.as_ref())?),
            InternableString::from(struct_upd.field.as_ref().req()?),
        ))
    }
}

pub type DamlRecUpdWrapper<'a> = PayloadElementWrapper<'a, &'a DamlRecUpdPayload<'a>>;

#[derive(Debug)]
pub struct DamlRecUpdPayload<'a> {
    pub tycon: DamlTyConPayload<'a>,
    pub record: Box<DamlExprPayload<'a>>,
    pub update: Box<DamlExprPayload<'a>>,
    pub field: InternableString<'a>,
}

impl<'a> DamlRecUpdPayload<'a> {
    pub fn new(
        tycon: DamlTyConPayload<'a>,
        record: Box<DamlExprPayload<'a>>,
        update: Box<DamlExprPayload<'a>>,
        field: InternableString<'a>,
    ) -> Self {
        Self {
            tycon,
            record,
            update,
            field,
        }
    }
}

impl<'a> TryFrom<&'a RecUpd> for DamlRecUpdPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(rec_upd: &'a RecUpd) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTyConPayload::try_from(rec_upd.tycon.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(rec_upd.record.as_ref().req()?.as_ref())?),
            Box::new(DamlExprPayload::try_from(rec_upd.update.as_ref().req()?.as_ref())?),
            InternableString::from(rec_upd.field.as_ref().req()?),
        ))
    }
}

pub type DamlStructProjWrapper<'a> = PayloadElementWrapper<'a, &'a DamlStructProjPayload<'a>>;

#[derive(Debug)]
pub struct DamlStructProjPayload<'a> {
    pub struct_expr: Box<DamlExprPayload<'a>>,
    pub field: InternableString<'a>,
}

impl<'a> DamlStructProjPayload<'a> {
    pub fn new(struct_expr: Box<DamlExprPayload<'a>>, field: InternableString<'a>) -> Self {
        Self {
            struct_expr,
            field,
        }
    }
}

impl<'a> TryFrom<&'a StructProj> for DamlStructProjPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(struct_proj: &'a StructProj) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            Box::new(DamlExprPayload::try_from(struct_proj.r#struct.as_ref().req()?.as_ref())?),
            InternableString::from(struct_proj.field.as_ref().req()?),
        ))
    }
}

pub type DamlRecProjWrapper<'a> = PayloadElementWrapper<'a, &'a DamlRecProjPayload<'a>>;

#[derive(Debug)]
pub struct DamlRecProjPayload<'a> {
    pub tycon: DamlTyConPayload<'a>,
    pub record: Box<DamlExprPayload<'a>>,
    pub field: InternableString<'a>,
}

impl<'a> DamlRecProjPayload<'a> {
    pub fn new(tycon: DamlTyConPayload<'a>, record: Box<DamlExprPayload<'a>>, field: InternableString<'a>) -> Self {
        Self {
            tycon,
            record,
            field,
        }
    }
}

impl<'a> TryFrom<&'a RecProj> for DamlRecProjPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(rec_proj: &'a RecProj) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTyConPayload::try_from(rec_proj.tycon.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(rec_proj.record.as_ref().req()?.as_ref())?),
            InternableString::from(rec_proj.field.as_ref().req()?),
        ))
    }
}

pub type DamlConsWrapper<'a> = PayloadElementWrapper<'a, &'a DamlConsPayload<'a>>;

#[derive(Debug)]
pub struct DamlConsPayload<'a> {
    pub ty: DamlTypePayload<'a>,
    pub front: Vec<DamlExprPayload<'a>>,
    pub tail: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlConsPayload<'a> {
    pub fn new(ty: DamlTypePayload<'a>, front: Vec<DamlExprPayload<'a>>, tail: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            ty,
            front,
            tail,
        }
    }
}

impl<'a> TryFrom<&'a Cons> for DamlConsPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(cons: &'a Cons) -> DamlLfConvertResult<Self> {
        Ok(DamlConsPayload::new(
            DamlTypePayload::try_from(cons.r#type.as_ref().req()?)?,
            cons.front.iter().map(DamlExprPayload::try_from).collect::<DamlLfConvertResult<_>>()?,
            Box::new(DamlExprPayload::try_from(cons.tail.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlStructConWrapper<'a> = PayloadElementWrapper<'a, &'a DamlStructConPayload<'a>>;

#[derive(Debug)]
pub struct DamlStructConPayload<'a> {
    pub fields: Vec<DamlFieldWithExprPayload<'a>>,
}

impl<'a> DamlStructConPayload<'a> {
    pub fn new(fields: Vec<DamlFieldWithExprPayload<'a>>) -> Self {
        Self {
            fields,
        }
    }
}

impl<'a> TryFrom<&'a StructCon> for DamlStructConPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(struct_con: &'a StructCon) -> DamlLfConvertResult<Self> {
        Ok(DamlStructConPayload::new(
            struct_con.fields.iter().map(DamlFieldWithExprPayload::try_from).collect::<DamlLfConvertResult<_>>()?,
        ))
    }
}

pub type DamlEnumConWrapper<'a> = PayloadElementWrapper<'a, &'a DamlEnumConPayload<'a>>;

#[derive(Debug)]
pub struct DamlEnumConPayload<'a> {
    pub tycon: DamlTyConNamePayload<'a>,
    pub enum_con: InternableString<'a>,
}

impl<'a> DamlEnumConPayload<'a> {
    pub fn new(tycon: DamlTyConNamePayload<'a>, enum_con: InternableString<'a>) -> Self {
        Self {
            tycon,
            enum_con,
        }
    }
}

impl<'a> TryFrom<&'a EnumCon> for DamlEnumConPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(enum_con: &'a EnumCon) -> DamlLfConvertResult<Self> {
        let tycon = DamlTyConNamePayload::try_from(enum_con.tycon.as_ref().req()?)?;
        let enum_con = InternableString::from(enum_con.enum_con.as_ref().req()?);
        Ok(DamlEnumConPayload::new(tycon, enum_con))
    }
}

pub type DamlVariantConWrapper<'a> = PayloadElementWrapper<'a, &'a DamlVariantConPayload<'a>>;

#[derive(Debug)]
pub struct DamlVariantConPayload<'a> {
    pub tycon: DamlTyConPayload<'a>,
    pub variant_arg: Box<DamlExprPayload<'a>>,
    pub variant_con: InternableString<'a>,
}

impl<'a> DamlVariantConPayload<'a> {
    pub fn new(
        tycon: DamlTyConPayload<'a>,
        variant_arg: Box<DamlExprPayload<'a>>,
        variant_con: InternableString<'a>,
    ) -> Self {
        Self {
            tycon,
            variant_arg,
            variant_con,
        }
    }
}

impl<'a> TryFrom<&'a VariantCon> for DamlVariantConPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(variant_con: &'a VariantCon) -> DamlLfConvertResult<Self> {
        let tycon = DamlTyConPayload::try_from(variant_con.tycon.as_ref().req()?)?;
        let variant_arg = DamlExprPayload::try_from(variant_con.variant_arg.as_ref().req()?.as_ref())?;
        let variant_con = InternableString::from(variant_con.variant_con.as_ref().req()?);
        Ok(DamlVariantConPayload::new(tycon, Box::new(variant_arg), variant_con))
    }
}

pub type DamlRecConWrapper<'a> = PayloadElementWrapper<'a, &'a DamlRecConPayload<'a>>;

#[derive(Debug)]
pub struct DamlRecConPayload<'a> {
    pub tycon: DamlTyConPayload<'a>,
    pub fields: Vec<DamlFieldWithExprPayload<'a>>,
}

impl<'a> DamlRecConPayload<'a> {
    pub fn new(tycon: DamlTyConPayload<'a>, fields: Vec<DamlFieldWithExprPayload<'a>>) -> Self {
        Self {
            tycon,
            fields,
        }
    }
}

impl<'a> TryFrom<&'a RecCon> for DamlRecConPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(rec_con: &'a RecCon) -> DamlLfConvertResult<Self> {
        Ok(DamlRecConPayload::new(
            DamlTyConPayload::try_from(rec_con.tycon.as_ref().req()?)?,
            rec_con.fields.iter().map(DamlFieldWithExprPayload::try_from).collect::<DamlLfConvertResult<_>>()?,
        ))
    }
}

pub type DamlFieldWithExprWrapper<'a> = PayloadElementWrapper<'a, &'a DamlFieldWithExprPayload<'a>>;

#[derive(Debug)]
pub struct DamlFieldWithExprPayload<'a> {
    pub field: InternableString<'a>,
    pub expr: DamlExprPayload<'a>,
}

impl<'a> DamlFieldWithExprPayload<'a> {
    pub fn new(field: InternableString<'a>, expr: DamlExprPayload<'a>) -> Self {
        Self {
            field,
            expr,
        }
    }
}

impl<'a> TryFrom<&'a FieldWithExpr> for DamlFieldWithExprPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(field_with_expr: &'a FieldWithExpr) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            InternableString::from(field_with_expr.field.as_ref().req()?),
            DamlExprPayload::try_from(field_with_expr.expr.as_ref().req()?)?,
        ))
    }
}

pub type DamlPrimLitWrapper<'a> = PayloadElementWrapper<'a, &'a DamlPrimLitPayload<'a>>;

#[derive(Debug)]
pub enum DamlPrimLitPayload<'a> {
    Int64(i64),
    Text(InternableString<'a>),
    Party(InternableString<'a>),
    /// A LitDate represents the number of day since 1970-01-01 with allowed range from 0001-01-01 to 9999-12-31 and
    /// using a year-month-day format.
    Date(i32),
    /// A LitTimestamp represents the number of microseconds since 1970-01-01T00:00:00.000000Z with allowed range
    /// 0001-01-01T00:00:00.000000Z to 9999-12-31T23:59:59.999999Z using a
    /// year-month-day-hour-minute-second-microsecond format.
    Timestamp(i64),
    Numeric(InternableString<'a>),
}

impl<'a> TryFrom<&'a PrimLit> for DamlPrimLitPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(lit: &'a PrimLit) -> DamlLfConvertResult<Self> {
        Ok(match lit.sum.as_ref().req()? {
            prim_lit::Sum::Int64(i) => DamlPrimLitPayload::Int64(*i),
            prim_lit::Sum::TextStr(text) => DamlPrimLitPayload::Text(InternableString::LiteralString(text)),
            prim_lit::Sum::TextInternedStr(i) => DamlPrimLitPayload::Text(InternableString::InternedString(*i)),
            prim_lit::Sum::PartyStr(party) => DamlPrimLitPayload::Party(InternableString::LiteralString(party)),
            prim_lit::Sum::PartyInternedStr(i) => DamlPrimLitPayload::Party(InternableString::InternedString(*i)),
            prim_lit::Sum::Date(d) => DamlPrimLitPayload::Date(*d),
            prim_lit::Sum::Timestamp(ts) => DamlPrimLitPayload::Timestamp(*ts),
            prim_lit::Sum::DecimalStr(s) => DamlPrimLitPayload::Numeric(InternableString::LiteralString(s)),
            prim_lit::Sum::NumericInternedStr(i) => DamlPrimLitPayload::Numeric(InternableString::InternedString(*i)),
        })
    }
}

#[derive(Debug)]
pub enum DamlBuiltinFunctionPayload {
    AddDecimal,
    SubDecimal,
    MulDecimal,
    DivDecimal,
    RoundDecimal,
    AddNumeric,
    SubNumeric,
    MulNumeric,
    DivNumeric,
    RoundNumeric,
    CastNumeric,
    ShiftNumeric,
    AddInt64,
    SubInt64,
    MulInt64,
    DivInt64,
    ModInt64,
    ExpInt64,
    Foldl,
    Foldr,
    TextmapEmpty,
    TextmapInsert,
    TextmapLookup,
    TextmapDelete,
    TextmapToList,
    TextmapSize,
    ExplodeText,
    AppendText,
    Error,
    AnyExceptionMessage,
    MakeGeneralError,
    GeneralErrorMessage,
    MakeArithmeticError,
    ArithmeticErrorMessage,
    MakeContractError,
    ContractErrorMessage,
    LeqInt64,
    LeqDecimal,
    LeqNumeric,
    LeqText,
    LeqTimestamp,
    LeqDate,
    LeqParty,
    LessInt64,
    LessDecimal,
    LessNumeric,
    LessText,
    LessTimestamp,
    LessDate,
    LessParty,
    GeqInt64,
    GeqDecimal,
    GeqNumeric,
    GeqText,
    GeqTimestamp,
    GeqDate,
    GeqParty,
    GreaterInt64,
    GreaterDecimal,
    GreaterNumeric,
    GreaterText,
    GreaterTimestamp,
    GreaterDate,
    GreaterParty,
    ToTextInt64,
    ToTextDecimal,
    ToTextNumeric,
    ToTextText,
    ToTextTimestamp,
    ToTextDate,
    ToQuotedTextParty,
    ToTextParty,
    FromTextParty,
    FromTextInt64,
    FromTextDecimal,
    FromTextNumeric,
    ToTextContractId,
    Sha256Text,
    DateToUnixDays,
    UnixDaysToDate,
    TimestampToUnixMicroseconds,
    UnixMicrosecondsToTimestamp,
    Int64ToDecimal,
    DecimalToInt64,
    Int64ToNumeric,
    NumericToInt64,
    ImplodeText,
    EqualInt64,
    EqualDecimal,
    EqualNumeric,
    EqualText,
    EqualTimestamp,
    EqualDate,
    EqualParty,
    EqualBool,
    EqualContractId,
    EqualList,
    EqualTypeRep,
    Trace,
    CoerceContractId,
    TextFromCodePoints,
    TextToCodePoints,
    GenmapEmpty,
    GenmapInsert,
    GenmapLookup,
    GenmapDelete,
    GenmapKeys,
    GenmapValues,
    GenmapSize,
    Equal,
    LessEq,
    Less,
    GreaterEq,
    Greater,
    TextToUpper,
    TextToLower,
    TextSlice,
    TextSliceIndex,
    TextContainsOnly,
    TextReplicate,
    TextSplitOn,
    TextIntercalate,
}

impl<'a> TryFrom<&i32> for DamlBuiltinFunctionPayload {
    type Error = DamlLfConvertError;

    #[allow(clippy::too_many_lines)]
    fn try_from(builtin_function: &i32) -> DamlLfConvertResult<Self> {
        match BuiltinFunction::from_i32(*builtin_function) {
            Some(BuiltinFunction::AddDecimal) => Ok(DamlBuiltinFunctionPayload::AddDecimal),
            Some(BuiltinFunction::SubDecimal) => Ok(DamlBuiltinFunctionPayload::SubDecimal),
            Some(BuiltinFunction::MulDecimal) => Ok(DamlBuiltinFunctionPayload::MulDecimal),
            Some(BuiltinFunction::DivDecimal) => Ok(DamlBuiltinFunctionPayload::DivDecimal),
            Some(BuiltinFunction::RoundDecimal) => Ok(DamlBuiltinFunctionPayload::RoundDecimal),
            Some(BuiltinFunction::AddNumeric) => Ok(DamlBuiltinFunctionPayload::AddNumeric),
            Some(BuiltinFunction::SubNumeric) => Ok(DamlBuiltinFunctionPayload::SubNumeric),
            Some(BuiltinFunction::MulNumeric) => Ok(DamlBuiltinFunctionPayload::MulNumeric),
            Some(BuiltinFunction::DivNumeric) => Ok(DamlBuiltinFunctionPayload::DivNumeric),
            Some(BuiltinFunction::RoundNumeric) => Ok(DamlBuiltinFunctionPayload::RoundNumeric),
            Some(BuiltinFunction::CastNumeric) => Ok(DamlBuiltinFunctionPayload::CastNumeric),
            Some(BuiltinFunction::ShiftNumeric) => Ok(DamlBuiltinFunctionPayload::ShiftNumeric),
            Some(BuiltinFunction::AddInt64) => Ok(DamlBuiltinFunctionPayload::AddInt64),
            Some(BuiltinFunction::SubInt64) => Ok(DamlBuiltinFunctionPayload::SubInt64),
            Some(BuiltinFunction::MulInt64) => Ok(DamlBuiltinFunctionPayload::MulInt64),
            Some(BuiltinFunction::DivInt64) => Ok(DamlBuiltinFunctionPayload::DivInt64),
            Some(BuiltinFunction::ModInt64) => Ok(DamlBuiltinFunctionPayload::ModInt64),
            Some(BuiltinFunction::ExpInt64) => Ok(DamlBuiltinFunctionPayload::ExpInt64),
            Some(BuiltinFunction::Foldl) => Ok(DamlBuiltinFunctionPayload::Foldl),
            Some(BuiltinFunction::Foldr) => Ok(DamlBuiltinFunctionPayload::Foldr),
            Some(BuiltinFunction::TextmapEmpty) => Ok(DamlBuiltinFunctionPayload::TextmapEmpty),
            Some(BuiltinFunction::TextmapInsert) => Ok(DamlBuiltinFunctionPayload::TextmapInsert),
            Some(BuiltinFunction::TextmapLookup) => Ok(DamlBuiltinFunctionPayload::TextmapLookup),
            Some(BuiltinFunction::TextmapDelete) => Ok(DamlBuiltinFunctionPayload::TextmapDelete),
            Some(BuiltinFunction::TextmapToList) => Ok(DamlBuiltinFunctionPayload::TextmapToList),
            Some(BuiltinFunction::TextmapSize) => Ok(DamlBuiltinFunctionPayload::TextmapSize),
            Some(BuiltinFunction::ExplodeText) => Ok(DamlBuiltinFunctionPayload::ExplodeText),
            Some(BuiltinFunction::AppendText) => Ok(DamlBuiltinFunctionPayload::AppendText),
            Some(BuiltinFunction::Error) => Ok(DamlBuiltinFunctionPayload::Error),
            Some(BuiltinFunction::AnyExceptionMessage) => Ok(DamlBuiltinFunctionPayload::AnyExceptionMessage),
            Some(BuiltinFunction::MakeGeneralError) => Ok(DamlBuiltinFunctionPayload::MakeGeneralError),
            Some(BuiltinFunction::GeneralErrorMessage) => Ok(DamlBuiltinFunctionPayload::GeneralErrorMessage),
            Some(BuiltinFunction::MakeArithmeticError) => Ok(DamlBuiltinFunctionPayload::MakeArithmeticError),
            Some(BuiltinFunction::ArithmeticErrorMessage) => Ok(DamlBuiltinFunctionPayload::ArithmeticErrorMessage),
            Some(BuiltinFunction::MakeContractError) => Ok(DamlBuiltinFunctionPayload::MakeContractError),
            Some(BuiltinFunction::ContractErrorMessage) => Ok(DamlBuiltinFunctionPayload::ContractErrorMessage),
            Some(BuiltinFunction::LeqInt64) => Ok(DamlBuiltinFunctionPayload::LeqInt64),
            Some(BuiltinFunction::LeqDecimal) => Ok(DamlBuiltinFunctionPayload::LeqDecimal),
            Some(BuiltinFunction::LeqNumeric) => Ok(DamlBuiltinFunctionPayload::LeqNumeric),
            Some(BuiltinFunction::LeqText) => Ok(DamlBuiltinFunctionPayload::LeqText),
            Some(BuiltinFunction::LeqTimestamp) => Ok(DamlBuiltinFunctionPayload::LeqTimestamp),
            Some(BuiltinFunction::LeqDate) => Ok(DamlBuiltinFunctionPayload::LeqDate),
            Some(BuiltinFunction::LeqParty) => Ok(DamlBuiltinFunctionPayload::LeqParty),
            Some(BuiltinFunction::LessInt64) => Ok(DamlBuiltinFunctionPayload::LessInt64),
            Some(BuiltinFunction::LessDecimal) => Ok(DamlBuiltinFunctionPayload::LessDecimal),
            Some(BuiltinFunction::LessNumeric) => Ok(DamlBuiltinFunctionPayload::LessNumeric),
            Some(BuiltinFunction::LessText) => Ok(DamlBuiltinFunctionPayload::LessText),
            Some(BuiltinFunction::LessTimestamp) => Ok(DamlBuiltinFunctionPayload::LessTimestamp),
            Some(BuiltinFunction::LessDate) => Ok(DamlBuiltinFunctionPayload::LessDate),
            Some(BuiltinFunction::LessParty) => Ok(DamlBuiltinFunctionPayload::LessParty),
            Some(BuiltinFunction::GeqInt64) => Ok(DamlBuiltinFunctionPayload::GeqInt64),
            Some(BuiltinFunction::GeqDecimal) => Ok(DamlBuiltinFunctionPayload::GeqDecimal),
            Some(BuiltinFunction::GeqNumeric) => Ok(DamlBuiltinFunctionPayload::GeqNumeric),
            Some(BuiltinFunction::GeqText) => Ok(DamlBuiltinFunctionPayload::GeqText),
            Some(BuiltinFunction::GeqTimestamp) => Ok(DamlBuiltinFunctionPayload::GeqTimestamp),
            Some(BuiltinFunction::GeqDate) => Ok(DamlBuiltinFunctionPayload::GeqDate),
            Some(BuiltinFunction::GeqParty) => Ok(DamlBuiltinFunctionPayload::GeqParty),
            Some(BuiltinFunction::GreaterInt64) => Ok(DamlBuiltinFunctionPayload::GreaterInt64),
            Some(BuiltinFunction::GreaterDecimal) => Ok(DamlBuiltinFunctionPayload::GreaterDecimal),
            Some(BuiltinFunction::GreaterNumeric) => Ok(DamlBuiltinFunctionPayload::GreaterNumeric),
            Some(BuiltinFunction::GreaterText) => Ok(DamlBuiltinFunctionPayload::GreaterText),
            Some(BuiltinFunction::GreaterTimestamp) => Ok(DamlBuiltinFunctionPayload::GreaterTimestamp),
            Some(BuiltinFunction::GreaterDate) => Ok(DamlBuiltinFunctionPayload::GreaterDate),
            Some(BuiltinFunction::GreaterParty) => Ok(DamlBuiltinFunctionPayload::GreaterParty),
            Some(BuiltinFunction::ToTextInt64) => Ok(DamlBuiltinFunctionPayload::ToTextInt64),
            Some(BuiltinFunction::ToTextDecimal) => Ok(DamlBuiltinFunctionPayload::ToTextDecimal),
            Some(BuiltinFunction::ToTextNumeric) => Ok(DamlBuiltinFunctionPayload::ToTextNumeric),
            Some(BuiltinFunction::ToTextText) => Ok(DamlBuiltinFunctionPayload::ToTextText),
            Some(BuiltinFunction::ToTextTimestamp) => Ok(DamlBuiltinFunctionPayload::ToTextTimestamp),
            Some(BuiltinFunction::ToTextDate) => Ok(DamlBuiltinFunctionPayload::ToTextDate),
            Some(BuiltinFunction::ToQuotedTextParty) => Ok(DamlBuiltinFunctionPayload::ToQuotedTextParty),
            Some(BuiltinFunction::ToTextParty) => Ok(DamlBuiltinFunctionPayload::ToTextParty),
            Some(BuiltinFunction::FromTextParty) => Ok(DamlBuiltinFunctionPayload::FromTextParty),
            Some(BuiltinFunction::FromTextInt64) => Ok(DamlBuiltinFunctionPayload::FromTextInt64),
            Some(BuiltinFunction::FromTextDecimal) => Ok(DamlBuiltinFunctionPayload::FromTextDecimal),
            Some(BuiltinFunction::FromTextNumeric) => Ok(DamlBuiltinFunctionPayload::FromTextNumeric),
            Some(BuiltinFunction::ToTextContractId) => Ok(DamlBuiltinFunctionPayload::ToTextContractId),
            Some(BuiltinFunction::Sha256Text) => Ok(DamlBuiltinFunctionPayload::Sha256Text),
            Some(BuiltinFunction::DateToUnixDays) => Ok(DamlBuiltinFunctionPayload::DateToUnixDays),
            Some(BuiltinFunction::UnixDaysToDate) => Ok(DamlBuiltinFunctionPayload::UnixDaysToDate),
            Some(BuiltinFunction::TimestampToUnixMicroseconds) =>
                Ok(DamlBuiltinFunctionPayload::TimestampToUnixMicroseconds),
            Some(BuiltinFunction::UnixMicrosecondsToTimestamp) =>
                Ok(DamlBuiltinFunctionPayload::UnixMicrosecondsToTimestamp),
            Some(BuiltinFunction::Int64ToDecimal) => Ok(DamlBuiltinFunctionPayload::Int64ToDecimal),
            Some(BuiltinFunction::DecimalToInt64) => Ok(DamlBuiltinFunctionPayload::DecimalToInt64),
            Some(BuiltinFunction::Int64ToNumeric) => Ok(DamlBuiltinFunctionPayload::Int64ToNumeric),
            Some(BuiltinFunction::NumericToInt64) => Ok(DamlBuiltinFunctionPayload::NumericToInt64),
            Some(BuiltinFunction::ImplodeText) => Ok(DamlBuiltinFunctionPayload::ImplodeText),
            Some(BuiltinFunction::EqualInt64) => Ok(DamlBuiltinFunctionPayload::EqualInt64),
            Some(BuiltinFunction::EqualDecimal) => Ok(DamlBuiltinFunctionPayload::EqualDecimal),
            Some(BuiltinFunction::EqualNumeric) => Ok(DamlBuiltinFunctionPayload::EqualNumeric),
            Some(BuiltinFunction::EqualText) => Ok(DamlBuiltinFunctionPayload::EqualText),
            Some(BuiltinFunction::EqualTimestamp) => Ok(DamlBuiltinFunctionPayload::EqualTimestamp),
            Some(BuiltinFunction::EqualDate) => Ok(DamlBuiltinFunctionPayload::EqualDate),
            Some(BuiltinFunction::EqualParty) => Ok(DamlBuiltinFunctionPayload::EqualParty),
            Some(BuiltinFunction::EqualBool) => Ok(DamlBuiltinFunctionPayload::EqualBool),
            Some(BuiltinFunction::EqualContractId) => Ok(DamlBuiltinFunctionPayload::EqualContractId),
            Some(BuiltinFunction::EqualList) => Ok(DamlBuiltinFunctionPayload::EqualList),
            Some(BuiltinFunction::EqualTypeRep) => Ok(DamlBuiltinFunctionPayload::EqualTypeRep),
            Some(BuiltinFunction::Trace) => Ok(DamlBuiltinFunctionPayload::Trace),
            Some(BuiltinFunction::CoerceContractId) => Ok(DamlBuiltinFunctionPayload::CoerceContractId),
            Some(BuiltinFunction::TextFromCodePoints) => Ok(DamlBuiltinFunctionPayload::TextFromCodePoints),
            Some(BuiltinFunction::TextToCodePoints) => Ok(DamlBuiltinFunctionPayload::TextToCodePoints),
            Some(BuiltinFunction::GenmapEmpty) => Ok(DamlBuiltinFunctionPayload::GenmapEmpty),
            Some(BuiltinFunction::GenmapInsert) => Ok(DamlBuiltinFunctionPayload::GenmapInsert),
            Some(BuiltinFunction::GenmapLookup) => Ok(DamlBuiltinFunctionPayload::GenmapLookup),
            Some(BuiltinFunction::GenmapDelete) => Ok(DamlBuiltinFunctionPayload::GenmapDelete),
            Some(BuiltinFunction::GenmapKeys) => Ok(DamlBuiltinFunctionPayload::GenmapKeys),
            Some(BuiltinFunction::GenmapValues) => Ok(DamlBuiltinFunctionPayload::GenmapValues),
            Some(BuiltinFunction::GenmapSize) => Ok(DamlBuiltinFunctionPayload::GenmapSize),
            Some(BuiltinFunction::Equal) => Ok(DamlBuiltinFunctionPayload::Equal),
            Some(BuiltinFunction::LessEq) => Ok(DamlBuiltinFunctionPayload::LessEq),
            Some(BuiltinFunction::Less) => Ok(DamlBuiltinFunctionPayload::Less),
            Some(BuiltinFunction::GreaterEq) => Ok(DamlBuiltinFunctionPayload::GreaterEq),
            Some(BuiltinFunction::Greater) => Ok(DamlBuiltinFunctionPayload::Greater),
            Some(BuiltinFunction::TextToUpper) => Ok(DamlBuiltinFunctionPayload::TextToUpper),
            Some(BuiltinFunction::TextToLower) => Ok(DamlBuiltinFunctionPayload::TextToLower),
            Some(BuiltinFunction::TextSlice) => Ok(DamlBuiltinFunctionPayload::TextSlice),
            Some(BuiltinFunction::TextSliceIndex) => Ok(DamlBuiltinFunctionPayload::TextSliceIndex),
            Some(BuiltinFunction::TextContainsOnly) => Ok(DamlBuiltinFunctionPayload::TextContainsOnly),
            Some(BuiltinFunction::TextReplicate) => Ok(DamlBuiltinFunctionPayload::TextReplicate),
            Some(BuiltinFunction::TextSplitOn) => Ok(DamlBuiltinFunctionPayload::TextSplitOn),
            Some(BuiltinFunction::TextIntercalate) => Ok(DamlBuiltinFunctionPayload::TextIntercalate),
            None => Err(DamlLfConvertError::UnknownBuiltinFunction(*builtin_function)),
        }
    }
}

#[derive(Debug)]
pub enum DamlPrimConPayload {
    Unit,
    False,
    True,
}

impl<'a> TryFrom<&i32> for DamlPrimConPayload {
    type Error = DamlLfConvertError;

    fn try_from(prim_con: &i32) -> DamlLfConvertResult<Self> {
        match PrimCon::from_i32(*prim_con) {
            Some(PrimCon::ConFalse) => Ok(DamlPrimConPayload::False),
            Some(PrimCon::ConTrue) => Ok(DamlPrimConPayload::True),
            Some(PrimCon::ConUnit) => Ok(DamlPrimConPayload::Unit),
            None => Err(DamlLfConvertError::UnknownPrimCon(*prim_con)),
        }
    }
}

///
pub type DamlValueNameWrapper<'a> = PayloadElementWrapper<'a, &'a DamlValueNamePayload<'a>>;

#[derive(Debug)]
pub struct DamlValueNamePayload<'a> {
    pub package_ref: DamlPackageRefPayload<'a>,
    pub module_path: InternableDottedName<'a>,
    pub name: InternableDottedName<'a>,
}

impl<'a> DamlValueNamePayload<'a> {
    pub fn new(
        package_ref: DamlPackageRefPayload<'a>,
        module_path: InternableDottedName<'a>,
        name: InternableDottedName<'a>,
    ) -> Self {
        Self {
            package_ref,
            module_path,
            name,
        }
    }
}

impl<'a> TryFrom<&'a ValName> for DamlValueNamePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(val_name: &'a ValName) -> DamlLfConvertResult<Self> {
        match val_name {
            ValName {
                module:
                    Some(ModuleRef {
                        package_ref: Some(package_ref),
                        module_name: Some(module_name),
                    }),
                name_dname,
                name_interned_dname,
            } => Ok(Self::new(
                DamlPackageRefPayload::try_from(package_ref)?,
                InternableDottedName::from(module_name),
                InternableDottedName::new_implied(*name_interned_dname, name_dname),
            )),
            _ => Err(DamlLfConvertError::MissingRequiredField),
        }
    }
}

pub type DamlPureWrapper<'a> = PayloadElementWrapper<'a, &'a DamlPurePayload<'a>>;

#[derive(Debug)]
pub struct DamlPurePayload<'a> {
    pub ty: DamlTypePayload<'a>,
    pub expr: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlPurePayload<'a> {
    pub fn new(ty: DamlTypePayload<'a>, expr: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            ty,
            expr,
        }
    }
}

impl<'a> TryFrom<&'a Pure> for DamlPurePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(pure: &'a Pure) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTypePayload::try_from(pure.r#type.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(pure.expr.as_ref().req()?.as_ref())?),
        ))
    }
}

///
pub type DamlUpdateWrapper<'a> = PayloadElementWrapper<'a, &'a DamlUpdatePayload<'a>>;

#[derive(Debug)]
pub enum DamlUpdatePayload<'a> {
    Pure(DamlPurePayload<'a>),
    Block(DamlBlockPayload<'a>),
    Create(DamlCreatePayload<'a>),
    Exercise(DamlExercisePayload<'a>),
    ExerciseByKey(DamlExerciseByKeyPayload<'a>),
    Fetch(DamlFetchPayload<'a>),
    GetTime,
    LookupByKey(DamlRetrieveByKeyPayload<'a>),
    FetchByKey(DamlRetrieveByKeyPayload<'a>),
    EmbedExpr(DamlUpdateEmbedExprPayload<'a>),
    TryCatch(DamlTryCatchPayload<'a>),
}

impl<'a> TryFrom<&'a Update> for DamlUpdatePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(update: &'a Update) -> DamlLfConvertResult<Self> {
        Ok(match update.sum.as_ref().req()? {
            update::Sum::Pure(pure) => DamlUpdatePayload::Pure(DamlPurePayload::try_from(pure.as_ref())?),
            update::Sum::Block(block) => DamlUpdatePayload::Block(DamlBlockPayload::try_from(block.as_ref())?),
            update::Sum::Create(create) => DamlUpdatePayload::Create(DamlCreatePayload::try_from(create.as_ref())?),
            update::Sum::Exercise(exercise) =>
                DamlUpdatePayload::Exercise(DamlExercisePayload::try_from(exercise.as_ref())?),
            update::Sum::ExerciseByKey(exercise_by_key) =>
                DamlUpdatePayload::ExerciseByKey(DamlExerciseByKeyPayload::try_from(exercise_by_key.as_ref())?),
            update::Sum::Fetch(fetch) => DamlUpdatePayload::Fetch(DamlFetchPayload::try_from(fetch.as_ref())?),
            update::Sum::GetTime(_) => DamlUpdatePayload::GetTime,
            update::Sum::LookupByKey(retrieve_by_key) =>
                DamlUpdatePayload::LookupByKey(DamlRetrieveByKeyPayload::try_from(retrieve_by_key.as_ref())?),
            update::Sum::FetchByKey(retrieve_by_key) =>
                DamlUpdatePayload::FetchByKey(DamlRetrieveByKeyPayload::try_from(retrieve_by_key.as_ref())?),
            update::Sum::EmbedExpr(embed_expr) =>
                DamlUpdatePayload::EmbedExpr(DamlUpdateEmbedExprPayload::try_from(embed_expr.as_ref())?),
            update::Sum::TryCatch(try_catch) =>
                DamlUpdatePayload::TryCatch(DamlTryCatchPayload::try_from(try_catch.as_ref())?),
        })
    }
}

pub type DamlUpdateEmbedExprWrapper<'a> = PayloadElementWrapper<'a, &'a DamlUpdateEmbedExprPayload<'a>>;

#[derive(Debug)]
pub struct DamlUpdateEmbedExprPayload<'a> {
    pub ty: DamlTypePayload<'a>,
    pub body: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlUpdateEmbedExprPayload<'a> {
    pub fn new(ty: DamlTypePayload<'a>, body: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            ty,
            body,
        }
    }
}

impl<'a> TryFrom<&'a update::EmbedExpr> for DamlUpdateEmbedExprPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(embed_expr: &'a update::EmbedExpr) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTypePayload::try_from(embed_expr.r#type.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(embed_expr.body.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlRetrieveByKeyWrapper<'a> = PayloadElementWrapper<'a, &'a DamlRetrieveByKeyPayload<'a>>;

#[derive(Debug)]
pub struct DamlRetrieveByKeyPayload<'a> {
    pub template: DamlTyConNamePayload<'a>,
    pub key: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlRetrieveByKeyPayload<'a> {
    pub fn new(template: DamlTyConNamePayload<'a>, key: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            template,
            key,
        }
    }
}

impl<'a> TryFrom<&'a RetrieveByKey> for DamlRetrieveByKeyPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(retrieve_by_key: &'a RetrieveByKey) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTyConNamePayload::try_from(retrieve_by_key.template.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(retrieve_by_key.key.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlFetchWrapper<'a> = PayloadElementWrapper<'a, &'a DamlFetchPayload<'a>>;

#[derive(Debug)]
pub struct DamlFetchPayload<'a> {
    pub template: DamlTyConNamePayload<'a>,
    pub cid: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlFetchPayload<'a> {
    pub fn new(template: DamlTyConNamePayload<'a>, cid: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            template,
            cid,
        }
    }
}

impl<'a> TryFrom<&'a Fetch> for DamlFetchPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(fetch: &'a Fetch) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTyConNamePayload::try_from(fetch.template.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(fetch.cid.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlExerciseWrapper<'a> = PayloadElementWrapper<'a, &'a DamlExercisePayload<'a>>;

#[derive(Debug)]
pub struct DamlExercisePayload<'a> {
    pub template: DamlTyConNamePayload<'a>,
    pub cid: Box<DamlExprPayload<'a>>,
    pub arg: Box<DamlExprPayload<'a>>,
    pub choice: InternableString<'a>,
}

impl<'a> DamlExercisePayload<'a> {
    pub fn new(
        template: DamlTyConNamePayload<'a>,
        cid: Box<DamlExprPayload<'a>>,
        arg: Box<DamlExprPayload<'a>>,
        choice: InternableString<'a>,
    ) -> Self {
        Self {
            template,
            cid,
            arg,
            choice,
        }
    }
}

impl<'a> TryFrom<&'a Exercise> for DamlExercisePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(exercise: &'a Exercise) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTyConNamePayload::try_from(exercise.template.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(exercise.cid.as_ref().req()?.as_ref())?),
            Box::new(DamlExprPayload::try_from(exercise.arg.as_ref().req()?.as_ref())?),
            InternableString::from(exercise.choice.as_ref().req()?),
        ))
    }
}

pub type DamlExerciseByKeyWrapper<'a> = PayloadElementWrapper<'a, &'a DamlExerciseByKeyPayload<'a>>;

#[derive(Debug)]
pub struct DamlExerciseByKeyPayload<'a> {
    pub template: DamlTyConNamePayload<'a>,
    pub choice: InternableString<'a>,
    pub key: Box<DamlExprPayload<'a>>,
    pub arg: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlExerciseByKeyPayload<'a> {
    pub fn new(
        template: DamlTyConNamePayload<'a>,
        choice: InternableString<'a>,
        key: Box<DamlExprPayload<'a>>,
        arg: Box<DamlExprPayload<'a>>,
    ) -> Self {
        Self {
            template,
            choice,
            key,
            arg,
        }
    }
}

impl<'a> TryFrom<&'a ExerciseByKey> for DamlExerciseByKeyPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(exercise_by_key: &'a ExerciseByKey) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTyConNamePayload::try_from(exercise_by_key.template.as_ref().req()?)?,
            InternableString::InternedString(exercise_by_key.choice_interned_str),
            Box::new(DamlExprPayload::try_from(exercise_by_key.key.as_ref().req()?.as_ref())?),
            Box::new(DamlExprPayload::try_from(exercise_by_key.arg.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlTryCatchWrapper<'a> = PayloadElementWrapper<'a, &'a DamlTryCatchPayload<'a>>;

#[derive(Debug)]
pub struct DamlTryCatchPayload<'a> {
    pub return_type: DamlTypePayload<'a>,
    pub try_expr: Box<DamlExprPayload<'a>>,
    pub var: InternableString<'a>,
    pub catch_expr: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlTryCatchPayload<'a> {
    pub fn new(
        return_type: DamlTypePayload<'a>,
        try_expr: Box<DamlExprPayload<'a>>,
        var: InternableString<'a>,
        catch_expr: Box<DamlExprPayload<'a>>,
    ) -> Self {
        Self {
            return_type,
            try_expr,
            var,
            catch_expr,
        }
    }
}

impl<'a> TryFrom<&'a TryCatch> for DamlTryCatchPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(try_catch: &'a TryCatch) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTypePayload::try_from(try_catch.return_type.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(try_catch.try_expr.as_ref().req()?.as_ref())?),
            InternableString::InternedString(try_catch.var_interned_str),
            Box::new(DamlExprPayload::try_from(try_catch.catch_expr.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlCreateWrapper<'a> = PayloadElementWrapper<'a, &'a DamlCreatePayload<'a>>;

#[derive(Debug)]
pub struct DamlCreatePayload<'a> {
    pub template: DamlTyConNamePayload<'a>,
    pub expr: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlCreatePayload<'a> {
    pub fn new(template: DamlTyConNamePayload<'a>, expr: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            template,
            expr,
        }
    }
}

impl<'a> TryFrom<&'a Create> for DamlCreatePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(create: &'a Create) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTyConNamePayload::try_from(create.template.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(create.expr.as_ref().req()?.as_ref())?),
        ))
    }
}

///
pub type DamlScenarioWrapper<'a> = PayloadElementWrapper<'a, &'a DamlScenarioPayload<'a>>;

#[derive(Debug)]
pub enum DamlScenarioPayload<'a> {
    Pure(DamlPurePayload<'a>),
    Block(DamlBlockPayload<'a>),
    Commit(DamlCommitPayload<'a>),
    MustFailAt(DamlCommitPayload<'a>),
    Pass(Box<DamlExprPayload<'a>>),
    GetTime,
    GetParty(Box<DamlExprPayload<'a>>),
    EmbedExpr(DamlScenarioEmbedExprPayload<'a>),
}

impl<'a> TryFrom<&'a Scenario> for DamlScenarioPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(scenario: &'a Scenario) -> DamlLfConvertResult<Self> {
        Ok(match scenario.sum.as_ref().req()? {
            scenario::Sum::Pure(pure) => DamlScenarioPayload::Pure(DamlPurePayload::try_from(pure.as_ref())?),
            scenario::Sum::Block(block) => DamlScenarioPayload::Block(DamlBlockPayload::try_from(block.as_ref())?),
            scenario::Sum::Commit(commit) | scenario::Sum::MustFailAt(commit) =>
                DamlScenarioPayload::Commit(DamlCommitPayload::try_from(commit.as_ref())?),
            scenario::Sum::Pass(embed_expr) =>
                DamlScenarioPayload::Pass(Box::new(DamlExprPayload::try_from(embed_expr.as_ref())?)),
            scenario::Sum::GetTime(_) => DamlScenarioPayload::GetTime,
            scenario::Sum::GetParty(expr) =>
                DamlScenarioPayload::GetParty(Box::new(DamlExprPayload::try_from(expr.as_ref())?)),
            scenario::Sum::EmbedExpr(embed_expr) =>
                DamlScenarioPayload::EmbedExpr(DamlScenarioEmbedExprPayload::try_from(embed_expr.as_ref())?),
        })
    }
}

pub type DamlCommitWrapper<'a> = PayloadElementWrapper<'a, &'a DamlCommitPayload<'a>>;

#[derive(Debug)]
pub struct DamlCommitPayload<'a> {
    pub party: Box<DamlExprPayload<'a>>,
    pub expr: Box<DamlExprPayload<'a>>,
    pub ret_type: DamlTypePayload<'a>,
}

impl<'a> DamlCommitPayload<'a> {
    pub fn new(party: Box<DamlExprPayload<'a>>, expr: Box<DamlExprPayload<'a>>, ret_type: DamlTypePayload<'a>) -> Self {
        Self {
            party,
            expr,
            ret_type,
        }
    }
}

impl<'a> TryFrom<&'a Commit> for DamlCommitPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(commit: &'a Commit) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            Box::new(DamlExprPayload::try_from(commit.party.as_ref().req()?.as_ref())?),
            Box::new(DamlExprPayload::try_from(commit.expr.as_ref().req()?.as_ref())?),
            DamlTypePayload::try_from(commit.ret_type.as_ref().req()?)?,
        ))
    }
}

pub type DamlScenarioEmbedExprWrapper<'a> = PayloadElementWrapper<'a, &'a DamlScenarioEmbedExprPayload<'a>>;

#[derive(Debug)]
pub struct DamlScenarioEmbedExprPayload<'a> {
    pub ty: DamlTypePayload<'a>,
    pub body: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlScenarioEmbedExprPayload<'a> {
    pub fn new(ty: DamlTypePayload<'a>, body: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            ty,
            body,
        }
    }
}

impl<'a> TryFrom<&'a scenario::EmbedExpr> for DamlScenarioEmbedExprPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(embed_expr: &'a scenario::EmbedExpr) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTypePayload::try_from(embed_expr.r#type.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(embed_expr.body.as_ref().req()?.as_ref())?),
        ))
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum DamlKeyExprPayload<'a> {
    LegacyKey,
    ComplexKey(DamlExprPayload<'a>),
}

impl<'a> TryFrom<&'a def_key::KeyExpr> for DamlKeyExprPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(key_expr: &'a def_key::KeyExpr) -> DamlLfConvertResult<Self> {
        Ok(match key_expr {
            def_key::KeyExpr::Key(_) => DamlKeyExprPayload::LegacyKey,
            def_key::KeyExpr::ComplexKey(expr) => DamlKeyExprPayload::ComplexKey(DamlExprPayload::try_from(expr)?),
        })
    }
}

pub type DamlToAnyExceptionWrapper<'a> = PayloadElementWrapper<'a, &'a DamlToAnyExceptionPayload<'a>>;

#[derive(Debug)]
pub struct DamlToAnyExceptionPayload<'a> {
    pub ty: DamlTypePayload<'a>,
    pub expr: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlToAnyExceptionPayload<'a> {
    pub fn new(ty: DamlTypePayload<'a>, expr: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            ty,
            expr,
        }
    }
}

impl<'a> TryFrom<&'a ToAnyException> for DamlToAnyExceptionPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(to_any_exception: &'a ToAnyException) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTypePayload::try_from(to_any_exception.r#type.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(to_any_exception.expr.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlFromAnyExceptionWrapper<'a> = PayloadElementWrapper<'a, &'a DamlFromAnyExceptionPayload<'a>>;

#[derive(Debug)]
pub struct DamlFromAnyExceptionPayload<'a> {
    pub ty: DamlTypePayload<'a>,
    pub expr: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlFromAnyExceptionPayload<'a> {
    pub fn new(ty: DamlTypePayload<'a>, expr: Box<DamlExprPayload<'a>>) -> Self {
        Self {
            ty,
            expr,
        }
    }
}

impl<'a> TryFrom<&'a FromAnyException> for DamlFromAnyExceptionPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(from_any_exception: &'a FromAnyException) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTypePayload::try_from(from_any_exception.r#type.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(from_any_exception.expr.as_ref().req()?.as_ref())?),
        ))
    }
}

pub type DamlThrowWrapper<'a> = PayloadElementWrapper<'a, &'a DamlThrowPayload<'a>>;

#[derive(Debug)]
pub struct DamlThrowPayload<'a> {
    pub return_type: DamlTypePayload<'a>,
    pub exception_type: DamlTypePayload<'a>,
    pub exception_expr: Box<DamlExprPayload<'a>>,
}

impl<'a> DamlThrowPayload<'a> {
    pub fn new(
        return_type: DamlTypePayload<'a>,
        exception_type: DamlTypePayload<'a>,
        exception_expr: Box<DamlExprPayload<'a>>,
    ) -> Self {
        Self {
            return_type,
            exception_type,
            exception_expr,
        }
    }
}

impl<'a> TryFrom<&'a Throw> for DamlThrowPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(throw: &'a Throw) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTypePayload::try_from(throw.return_type.as_ref().req()?)?,
            DamlTypePayload::try_from(throw.exception_type.as_ref().req()?)?,
            Box::new(DamlExprPayload::try_from(throw.exception_expr.as_ref().req()?.as_ref())?),
        ))
    }
}
