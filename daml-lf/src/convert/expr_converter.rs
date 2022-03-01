use crate::convert::defvalue_payload::DamlDefValueWrapper;
use crate::convert::expr_payload::{
    DamlAbsWrapper, DamlAppWrapper, DamlBindingWrapper, DamlBlockWrapper, DamlBuiltinFunctionPayload,
    DamlCaseAltConsWrapper, DamlCaseAltEnumWrapper, DamlCaseAltOptionalSomeWrapper, DamlCaseAltSumPayload,
    DamlCaseAltSumWrapper, DamlCaseAltVariantWrapper, DamlCaseAltWrapper, DamlCaseWrapper, DamlCommitWrapper,
    DamlConsWrapper, DamlCreateWrapper, DamlEnumConWrapper, DamlExerciseByKeyWrapper, DamlExerciseWrapper,
    DamlExprPayload, DamlExprWrapper, DamlFetchWrapper, DamlFieldWithExprWrapper, DamlFromAnyExceptionWrapper,
    DamlFromAnyWrapper, DamlOptionalSomeWrapper, DamlPrimConPayload, DamlPrimLitPayload, DamlPrimLitWrapper,
    DamlPureWrapper, DamlRecConWrapper, DamlRecProjWrapper, DamlRecUpdWrapper, DamlRetrieveByKeyWrapper,
    DamlScenarioEmbedExprWrapper, DamlScenarioPayload, DamlScenarioWrapper, DamlStructConWrapper,
    DamlStructProjWrapper, DamlStructUpdWrapper, DamlThrowWrapper, DamlToAnyExceptionWrapper, DamlToAnyWrapper,
    DamlTryCatchWrapper, DamlTyAbsWrapper, DamlTyAppWrapper, DamlUpdateEmbedExprWrapper, DamlUpdatePayload,
    DamlUpdateWrapper, DamlValueNameWrapper, DamlVarWithTypeWrapper, DamlVariantConWrapper, RoundingModePayload,
};
use crate::convert::package_payload::DamlPackagePayload;
use crate::element::{
    DamlAbs, DamlApp, DamlBinding, DamlBlock, DamlBuiltinFunction, DamlCase, DamlCaseAlt, DamlCaseAltCons,
    DamlCaseAltEnum, DamlCaseAltOptionalSome, DamlCaseAltSum, DamlCaseAltVariant, DamlCommit, DamlCons, DamlCreate,
    DamlDefValue, DamlEnumCon, DamlExercise, DamlExerciseByKey, DamlExpr, DamlFetch, DamlFieldWithExpr, DamlFromAny,
    DamlFromAnyException, DamlLocalValueName, DamlNonLocalValueName, DamlOptionalSome, DamlPrimCon, DamlPrimLit,
    DamlPure, DamlRecCon, DamlRecProj, DamlRecUpd, DamlRetrieveByKey, DamlScenario, DamlScenarioEmbedExpr,
    DamlStructCon, DamlStructProj, DamlStructUpd, DamlThrow, DamlToAny, DamlToAnyException, DamlTryCatch, DamlTyAbs,
    DamlTyApp, DamlTyCon, DamlTyConName, DamlType, DamlTypeVarWithKind, DamlUpdate, DamlUpdateEmbedExpr, DamlValueName,
    DamlVarWithType, DamlVariantCon, RoundingMode,
};
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use std::borrow::Cow;
use std::convert::TryFrom;

/// Convert from `DamlDefValueWrapper` to `DamlDefValue`.
impl<'a> TryFrom<&DamlDefValueWrapper<'a>> for DamlDefValue<'a> {
    type Error = DamlLfConvertError;

    fn try_from(def_value: &DamlDefValueWrapper<'a>) -> DamlLfConvertResult<Self> {
        Ok(DamlDefValue::new(
            def_value.payload.name.resolve_last(def_value.context.package)?,
            DamlType::try_from(&def_value.wrap(&def_value.payload.ty))?,
            DamlExpr::try_from(&def_value.wrap(&def_value.payload.expr))?,
            def_value.payload.no_party_literals,
            def_value.payload.is_test,
        ))
    }
}

/// Convert from `DamlExprWrapper` to `DamlExpr`.
impl<'a> TryFrom<&DamlExprWrapper<'a>> for DamlExpr<'a> {
    type Error = DamlLfConvertError;

    fn try_from(expr: &DamlExprWrapper<'a>) -> DamlLfConvertResult<Self> {
        Ok(match expr.payload {
            DamlExprPayload::Var(var) => DamlExpr::Var(var.resolve(expr.context.package)?),
            DamlExprPayload::Val(val) => DamlExpr::Val(Box::new(DamlValueName::try_from(&expr.wrap(val))?)),
            DamlExprPayload::Builtin(builtin) => DamlExpr::Builtin(DamlBuiltinFunction::from(builtin)),
            DamlExprPayload::PrimCon(prim_con) => DamlExpr::PrimCon(DamlPrimCon::try_from(prim_con)?),
            DamlExprPayload::PrimLit(prim_lit) => DamlExpr::PrimLit(DamlPrimLit::try_from(&expr.wrap(prim_lit))?),
            DamlExprPayload::RecCon(rec_con) => DamlExpr::RecCon(DamlRecCon::try_from(&expr.wrap(rec_con))?),
            DamlExprPayload::RecProj(rec_proj) => DamlExpr::RecProj(DamlRecProj::try_from(&expr.wrap(rec_proj))?),
            DamlExprPayload::RecUpd(rec_upd) => DamlExpr::RecUpd(DamlRecUpd::try_from(&expr.wrap(rec_upd))?),
            DamlExprPayload::VariantCon(variant_con) =>
                DamlExpr::VariantCon(DamlVariantCon::try_from(&expr.wrap(variant_con))?),
            DamlExprPayload::EnumCon(enum_con) => DamlExpr::EnumCon(DamlEnumCon::try_from(&expr.wrap(enum_con))?),
            DamlExprPayload::StructCon(struct_con) =>
                DamlExpr::StructCon(DamlStructCon::try_from(&expr.wrap(struct_con))?),
            DamlExprPayload::StructProj(struct_proj) =>
                DamlExpr::StructProj(DamlStructProj::try_from(&expr.wrap(struct_proj))?),
            DamlExprPayload::StructUpd(struct_upd) =>
                DamlExpr::StructUpd(DamlStructUpd::try_from(&expr.wrap(struct_upd))?),
            DamlExprPayload::App(app) => DamlExpr::App(DamlApp::try_from(&expr.wrap(app))?),
            DamlExprPayload::TyApp(ty_app) => DamlExpr::TyApp(DamlTyApp::try_from(&expr.wrap(ty_app))?),
            DamlExprPayload::Abs(abs) => DamlExpr::Abs(DamlAbs::try_from(&expr.wrap(abs))?),
            DamlExprPayload::TyAbs(ty_abs) => DamlExpr::TyAbs(DamlTyAbs::try_from(&expr.wrap(ty_abs))?),
            DamlExprPayload::Case(case) => DamlExpr::Case(DamlCase::try_from(&expr.wrap(case))?),
            DamlExprPayload::Let(block) => DamlExpr::Let(DamlBlock::try_from(&expr.wrap(block))?),
            DamlExprPayload::Nil(ty) => DamlExpr::Nil(DamlType::try_from(&expr.wrap(ty))?),
            DamlExprPayload::Cons(cons) => DamlExpr::Cons(DamlCons::try_from(&expr.wrap(cons))?),
            DamlExprPayload::Update(upd) => DamlExpr::Update(DamlUpdate::try_from(&expr.wrap(upd))?),
            DamlExprPayload::Scenario(scenario) => DamlExpr::Scenario(DamlScenario::try_from(&expr.wrap(scenario))?),
            DamlExprPayload::OptionalNone(ty) => DamlExpr::OptionalNone(DamlType::try_from(&expr.wrap(ty))?),
            DamlExprPayload::OptionalSome(opt_some) =>
                DamlExpr::OptionalSome(DamlOptionalSome::try_from(&expr.wrap(opt_some))?),
            DamlExprPayload::ToAny(to_any) => DamlExpr::ToAny(DamlToAny::try_from(&expr.wrap(to_any))?),
            DamlExprPayload::FromAny(from_any) => DamlExpr::FromAny(DamlFromAny::try_from(&expr.wrap(from_any))?),
            DamlExprPayload::TypeRep(ty) => DamlExpr::TypeRep(DamlType::try_from(&expr.wrap(ty))?),
            DamlExprPayload::ToAnyException(to_any_exception) =>
                DamlExpr::ToAnyException(DamlToAnyException::try_from(&expr.wrap(to_any_exception))?),
            DamlExprPayload::FromAnyException(from_any_exception) =>
                DamlExpr::FromAnyException(DamlFromAnyException::try_from(&expr.wrap(from_any_exception))?),
            DamlExprPayload::Throw(throw) => DamlExpr::Throw(DamlThrow::try_from(&expr.wrap(throw))?),
        })
    }
}

impl<'a> From<&DamlBuiltinFunctionPayload> for DamlBuiltinFunction {
    #[allow(clippy::too_many_lines)]
    fn from(builtin: &DamlBuiltinFunctionPayload) -> Self {
        match builtin {
            DamlBuiltinFunctionPayload::AddDecimal => DamlBuiltinFunction::AddDecimal,
            DamlBuiltinFunctionPayload::SubDecimal => DamlBuiltinFunction::SubDecimal,
            DamlBuiltinFunctionPayload::MulDecimal => DamlBuiltinFunction::MulDecimal,
            DamlBuiltinFunctionPayload::DivDecimal => DamlBuiltinFunction::DivDecimal,
            DamlBuiltinFunctionPayload::RoundDecimal => DamlBuiltinFunction::RoundDecimal,
            DamlBuiltinFunctionPayload::AddNumeric => DamlBuiltinFunction::AddNumeric,
            DamlBuiltinFunctionPayload::SubNumeric => DamlBuiltinFunction::SubNumeric,
            DamlBuiltinFunctionPayload::MulNumeric => DamlBuiltinFunction::MulNumeric,
            DamlBuiltinFunctionPayload::DivNumeric => DamlBuiltinFunction::DivNumeric,
            DamlBuiltinFunctionPayload::RoundNumeric => DamlBuiltinFunction::RoundNumeric,
            DamlBuiltinFunctionPayload::CastNumeric => DamlBuiltinFunction::CastNumeric,
            DamlBuiltinFunctionPayload::ShiftNumeric => DamlBuiltinFunction::ShiftNumeric,
            DamlBuiltinFunctionPayload::AddInt64 => DamlBuiltinFunction::AddInt64,
            DamlBuiltinFunctionPayload::SubInt64 => DamlBuiltinFunction::SubInt64,
            DamlBuiltinFunctionPayload::MulInt64 => DamlBuiltinFunction::MulInt64,
            DamlBuiltinFunctionPayload::DivInt64 => DamlBuiltinFunction::DivInt64,
            DamlBuiltinFunctionPayload::ModInt64 => DamlBuiltinFunction::ModInt64,
            DamlBuiltinFunctionPayload::ExpInt64 => DamlBuiltinFunction::ExpInt64,
            DamlBuiltinFunctionPayload::Foldl => DamlBuiltinFunction::Foldl,
            DamlBuiltinFunctionPayload::Foldr => DamlBuiltinFunction::Foldr,
            DamlBuiltinFunctionPayload::TextmapEmpty => DamlBuiltinFunction::TextmapEmpty,
            DamlBuiltinFunctionPayload::TextmapInsert => DamlBuiltinFunction::TextmapInsert,
            DamlBuiltinFunctionPayload::TextmapLookup => DamlBuiltinFunction::TextmapLookup,
            DamlBuiltinFunctionPayload::TextmapDelete => DamlBuiltinFunction::TextmapDelete,
            DamlBuiltinFunctionPayload::TextmapToList => DamlBuiltinFunction::TextmapToList,
            DamlBuiltinFunctionPayload::TextmapSize => DamlBuiltinFunction::TextmapSize,
            DamlBuiltinFunctionPayload::ExplodeText => DamlBuiltinFunction::ExplodeText,
            DamlBuiltinFunctionPayload::AppendText => DamlBuiltinFunction::AppendText,
            DamlBuiltinFunctionPayload::Error => DamlBuiltinFunction::Error,
            DamlBuiltinFunctionPayload::AnyExceptionMessage => DamlBuiltinFunction::AnyExceptionMessage,
            DamlBuiltinFunctionPayload::LeqInt64 => DamlBuiltinFunction::LeqInt64,
            DamlBuiltinFunctionPayload::LeqDecimal => DamlBuiltinFunction::LeqDecimal,
            DamlBuiltinFunctionPayload::LeqNumeric => DamlBuiltinFunction::LeqNumeric,
            DamlBuiltinFunctionPayload::LeqText => DamlBuiltinFunction::LeqText,
            DamlBuiltinFunctionPayload::LeqTimestamp => DamlBuiltinFunction::LeqTimestamp,
            DamlBuiltinFunctionPayload::LeqDate => DamlBuiltinFunction::LeqDate,
            DamlBuiltinFunctionPayload::LeqParty => DamlBuiltinFunction::LeqParty,
            DamlBuiltinFunctionPayload::LessInt64 => DamlBuiltinFunction::LessInt64,
            DamlBuiltinFunctionPayload::LessDecimal => DamlBuiltinFunction::LessDecimal,
            DamlBuiltinFunctionPayload::LessNumeric => DamlBuiltinFunction::LessNumeric,
            DamlBuiltinFunctionPayload::LessText => DamlBuiltinFunction::LessText,
            DamlBuiltinFunctionPayload::LessTimestamp => DamlBuiltinFunction::LessTimestamp,
            DamlBuiltinFunctionPayload::LessDate => DamlBuiltinFunction::LessDate,
            DamlBuiltinFunctionPayload::LessParty => DamlBuiltinFunction::LessParty,
            DamlBuiltinFunctionPayload::GeqInt64 => DamlBuiltinFunction::GeqInt64,
            DamlBuiltinFunctionPayload::GeqDecimal => DamlBuiltinFunction::GeqDecimal,
            DamlBuiltinFunctionPayload::GeqNumeric => DamlBuiltinFunction::GeqNumeric,
            DamlBuiltinFunctionPayload::GeqText => DamlBuiltinFunction::GeqText,
            DamlBuiltinFunctionPayload::GeqTimestamp => DamlBuiltinFunction::GeqTimestamp,
            DamlBuiltinFunctionPayload::GeqDate => DamlBuiltinFunction::GeqDate,
            DamlBuiltinFunctionPayload::GeqParty => DamlBuiltinFunction::GeqParty,
            DamlBuiltinFunctionPayload::GreaterInt64 => DamlBuiltinFunction::GreaterInt64,
            DamlBuiltinFunctionPayload::GreaterDecimal => DamlBuiltinFunction::GreaterDecimal,
            DamlBuiltinFunctionPayload::GreaterNumeric => DamlBuiltinFunction::GreaterNumeric,
            DamlBuiltinFunctionPayload::GreaterText => DamlBuiltinFunction::GreaterText,
            DamlBuiltinFunctionPayload::GreaterTimestamp => DamlBuiltinFunction::GreaterTimestamp,
            DamlBuiltinFunctionPayload::GreaterDate => DamlBuiltinFunction::GreaterDate,
            DamlBuiltinFunctionPayload::GreaterParty => DamlBuiltinFunction::GreaterParty,
            DamlBuiltinFunctionPayload::Int64ToText => DamlBuiltinFunction::Int64ToText,
            DamlBuiltinFunctionPayload::DecimalToText => DamlBuiltinFunction::DecimalToText,
            DamlBuiltinFunctionPayload::NumericToText => DamlBuiltinFunction::NumericToText,
            DamlBuiltinFunctionPayload::TextToText => DamlBuiltinFunction::TextToText,
            DamlBuiltinFunctionPayload::TimestampToText => DamlBuiltinFunction::TimestampToText,
            DamlBuiltinFunctionPayload::DateToText => DamlBuiltinFunction::DateToText,
            DamlBuiltinFunctionPayload::PartyToQuotedText => DamlBuiltinFunction::PartyToQuotedText,
            DamlBuiltinFunctionPayload::PartyToText => DamlBuiltinFunction::PartyToText,
            DamlBuiltinFunctionPayload::TextToParty => DamlBuiltinFunction::TextToParty,
            DamlBuiltinFunctionPayload::TextToInt64 => DamlBuiltinFunction::TextToInt64,
            DamlBuiltinFunctionPayload::TextToDecimal => DamlBuiltinFunction::TextToDecimal,
            DamlBuiltinFunctionPayload::TextToNumeric => DamlBuiltinFunction::TextToNumeric,
            DamlBuiltinFunctionPayload::ContractIdToText => DamlBuiltinFunction::ContractIdToText,
            DamlBuiltinFunctionPayload::Sha256Text => DamlBuiltinFunction::Sha256Text,
            DamlBuiltinFunctionPayload::DateToUnixDays => DamlBuiltinFunction::DateToUnixDays,
            DamlBuiltinFunctionPayload::UnixDaysToDate => DamlBuiltinFunction::UnixDaysToDate,
            DamlBuiltinFunctionPayload::TimestampToUnixMicroseconds => DamlBuiltinFunction::TimestampToUnixMicroseconds,
            DamlBuiltinFunctionPayload::UnixMicrosecondsToTimestamp => DamlBuiltinFunction::UnixMicrosecondsToTimestamp,
            DamlBuiltinFunctionPayload::Int64ToDecimal => DamlBuiltinFunction::Int64ToDecimal,
            DamlBuiltinFunctionPayload::DecimalToInt64 => DamlBuiltinFunction::DecimalToInt64,
            DamlBuiltinFunctionPayload::Int64ToNumeric => DamlBuiltinFunction::Int64ToNumeric,
            DamlBuiltinFunctionPayload::NumericToInt64 => DamlBuiltinFunction::NumericToInt64,
            DamlBuiltinFunctionPayload::ImplodeText => DamlBuiltinFunction::ImplodeText,
            DamlBuiltinFunctionPayload::EqualInt64 => DamlBuiltinFunction::EqualInt64,
            DamlBuiltinFunctionPayload::EqualDecimal => DamlBuiltinFunction::EqualDecimal,
            DamlBuiltinFunctionPayload::EqualNumeric => DamlBuiltinFunction::EqualNumeric,
            DamlBuiltinFunctionPayload::EqualText => DamlBuiltinFunction::EqualText,
            DamlBuiltinFunctionPayload::EqualTimestamp => DamlBuiltinFunction::EqualTimestamp,
            DamlBuiltinFunctionPayload::EqualDate => DamlBuiltinFunction::EqualDate,
            DamlBuiltinFunctionPayload::EqualParty => DamlBuiltinFunction::EqualParty,
            DamlBuiltinFunctionPayload::EqualBool => DamlBuiltinFunction::EqualBool,
            DamlBuiltinFunctionPayload::EqualContractId => DamlBuiltinFunction::EqualContractId,
            DamlBuiltinFunctionPayload::EqualList => DamlBuiltinFunction::EqualList,
            DamlBuiltinFunctionPayload::EqualTypeRep => DamlBuiltinFunction::EqualTypeRep,
            DamlBuiltinFunctionPayload::Trace => DamlBuiltinFunction::Trace,
            DamlBuiltinFunctionPayload::CoerceContractId => DamlBuiltinFunction::CoerceContractId,
            DamlBuiltinFunctionPayload::CodePointsToText => DamlBuiltinFunction::CodePointsToText,
            DamlBuiltinFunctionPayload::TextPointsToCode => DamlBuiltinFunction::TextPointsToCode,
            DamlBuiltinFunctionPayload::GenmapEmpty => DamlBuiltinFunction::GenmapEmpty,
            DamlBuiltinFunctionPayload::GenmapInsert => DamlBuiltinFunction::GenmapInsert,
            DamlBuiltinFunctionPayload::GenmapLookup => DamlBuiltinFunction::GenmapLookup,
            DamlBuiltinFunctionPayload::GenmapDelete => DamlBuiltinFunction::GenmapDelete,
            DamlBuiltinFunctionPayload::GenmapKeys => DamlBuiltinFunction::GenmapKeys,
            DamlBuiltinFunctionPayload::GenmapValues => DamlBuiltinFunction::GenmapValues,
            DamlBuiltinFunctionPayload::GenmapSize => DamlBuiltinFunction::GenmapSize,
            DamlBuiltinFunctionPayload::Equal => DamlBuiltinFunction::Equal,
            DamlBuiltinFunctionPayload::LessEq => DamlBuiltinFunction::LessEq,
            DamlBuiltinFunctionPayload::Less => DamlBuiltinFunction::Less,
            DamlBuiltinFunctionPayload::GreaterEq => DamlBuiltinFunction::GreaterEq,
            DamlBuiltinFunctionPayload::Greater => DamlBuiltinFunction::Greater,
            DamlBuiltinFunctionPayload::ScaleBignumeric => DamlBuiltinFunction::ScaleBignumeric,
            DamlBuiltinFunctionPayload::PrecisionBignumeric => DamlBuiltinFunction::PrecisionBignumeric,
            DamlBuiltinFunctionPayload::AddBignumeric => DamlBuiltinFunction::AddBignumeric,
            DamlBuiltinFunctionPayload::SubBignumeric => DamlBuiltinFunction::SubBignumeric,
            DamlBuiltinFunctionPayload::MulBignumeric => DamlBuiltinFunction::MulBignumeric,
            DamlBuiltinFunctionPayload::DivBignumeric => DamlBuiltinFunction::DivBignumeric,
            DamlBuiltinFunctionPayload::ShiftBignumeric => DamlBuiltinFunction::ShiftBignumeric,
            DamlBuiltinFunctionPayload::ShiftRightBignumeric => DamlBuiltinFunction::ShiftRightBignumeric,
            DamlBuiltinFunctionPayload::BigNumericToNumeric => DamlBuiltinFunction::BigNumericToNumeric,
            DamlBuiltinFunctionPayload::NumericToBigNumeric => DamlBuiltinFunction::NumericToBigNumeric,
            DamlBuiltinFunctionPayload::BigNumericToText => DamlBuiltinFunction::BigNumericToText,
        }
    }
}

impl<'a> TryFrom<&DamlPrimConPayload> for DamlPrimCon {
    type Error = DamlLfConvertError;

    fn try_from(prim_con: &DamlPrimConPayload) -> DamlLfConvertResult<Self> {
        Ok(match prim_con {
            DamlPrimConPayload::Unit => DamlPrimCon::Unit,
            DamlPrimConPayload::False => DamlPrimCon::False,
            DamlPrimConPayload::True => DamlPrimCon::True,
        })
    }
}

impl<'a> TryFrom<&DamlPrimLitWrapper<'a>> for DamlPrimLit<'a> {
    type Error = DamlLfConvertError;

    fn try_from(prim_con: &DamlPrimLitWrapper<'a>) -> DamlLfConvertResult<Self> {
        let resolver = prim_con.context.package;
        Ok(match prim_con.payload {
            DamlPrimLitPayload::Int64(i) => DamlPrimLit::Int64(*i),
            DamlPrimLitPayload::Text(text) => DamlPrimLit::Text(text.resolve(resolver)?),
            DamlPrimLitPayload::Party(party) => DamlPrimLit::Party(party.resolve(resolver)?),
            DamlPrimLitPayload::Date(date) => DamlPrimLit::Date(*date),
            DamlPrimLitPayload::Timestamp(timestamp) => DamlPrimLit::Timestamp(*timestamp),
            DamlPrimLitPayload::Numeric(numeric) => DamlPrimLit::Numeric(numeric.resolve(resolver)?),
            DamlPrimLitPayload::RoundingMode(mode) => DamlPrimLit::RoundingMode(RoundingMode::from(mode)),
        })
    }
}

impl<'a> From<&RoundingModePayload> for RoundingMode {
    fn from(rounding_mode: &RoundingModePayload) -> Self {
        match rounding_mode {
            RoundingModePayload::Up => RoundingMode::Up,
            RoundingModePayload::Down => RoundingMode::Down,
            RoundingModePayload::Ceiling => RoundingMode::Ceiling,
            RoundingModePayload::Floor => RoundingMode::Floor,
            RoundingModePayload::HalfUp => RoundingMode::HalfUp,
            RoundingModePayload::HalfDown => RoundingMode::HalfDown,
            RoundingModePayload::HalfEven => RoundingMode::HalfEven,
            RoundingModePayload::Unnecessary => RoundingMode::Unnecessary,
        }
    }
}

impl<'a> TryFrom<&DamlRecConWrapper<'a>> for DamlRecCon<'a> {
    type Error = DamlLfConvertError;

    fn try_from(rec_con: &DamlRecConWrapper<'a>) -> DamlLfConvertResult<Self> {
        let tycon = DamlTyCon::try_from(&rec_con.wrap(&rec_con.payload.tycon))?;
        let fields = rec_con
            .payload
            .fields
            .iter()
            .map(|field| DamlFieldWithExpr::try_from(&rec_con.wrap(field)))
            .collect::<DamlLfConvertResult<_>>()?;
        Ok(DamlRecCon::new(tycon, fields))
    }
}

impl<'a> TryFrom<&DamlFieldWithExprWrapper<'a>> for DamlFieldWithExpr<'a> {
    type Error = DamlLfConvertError;

    fn try_from(field_with_expr: &DamlFieldWithExprWrapper<'a>) -> DamlLfConvertResult<Self> {
        let field = field_with_expr.payload.field.resolve(field_with_expr.context.package)?;
        let expr = DamlExpr::try_from(&field_with_expr.wrap(&field_with_expr.payload.expr))?;
        Ok(DamlFieldWithExpr::new(field, expr))
    }
}

impl<'a> TryFrom<&DamlRecProjWrapper<'a>> for DamlRecProj<'a> {
    type Error = DamlLfConvertError;

    fn try_from(rec_proj: &DamlRecProjWrapper<'a>) -> DamlLfConvertResult<Self> {
        let tycon = DamlTyCon::try_from(&rec_proj.wrap(&rec_proj.payload.tycon))?;
        let record = DamlExpr::try_from(&rec_proj.wrap(rec_proj.payload.record.as_ref()))?;
        let field = rec_proj.payload.field.resolve(rec_proj.context.package)?;
        Ok(DamlRecProj::new(tycon, Box::new(record), field))
    }
}

impl<'a> TryFrom<&DamlRecUpdWrapper<'a>> for DamlRecUpd<'a> {
    type Error = DamlLfConvertError;

    fn try_from(rec_upd: &DamlRecUpdWrapper<'a>) -> DamlLfConvertResult<Self> {
        let tycon = DamlTyCon::try_from(&rec_upd.wrap(&rec_upd.payload.tycon))?;
        let record = DamlExpr::try_from(&rec_upd.wrap(rec_upd.payload.record.as_ref()))?;
        let update = DamlExpr::try_from(&rec_upd.wrap(rec_upd.payload.update.as_ref()))?;
        let field = rec_upd.payload.field.resolve(rec_upd.context.package)?;
        Ok(DamlRecUpd::new(tycon, Box::new(record), Box::new(update), field))
    }
}

impl<'a> TryFrom<&DamlVariantConWrapper<'a>> for DamlVariantCon<'a> {
    type Error = DamlLfConvertError;

    fn try_from(variant_con: &DamlVariantConWrapper<'a>) -> DamlLfConvertResult<Self> {
        let tycon = DamlTyCon::try_from(&variant_con.wrap(&variant_con.payload.tycon))?;
        let variant_arg = DamlExpr::try_from(&variant_con.wrap(variant_con.payload.variant_arg.as_ref()))?;
        let variant_con = variant_con.payload.variant_con.resolve(variant_con.context.package)?;
        Ok(DamlVariantCon::new(tycon, Box::new(variant_arg), variant_con))
    }
}

impl<'a> TryFrom<&DamlEnumConWrapper<'a>> for DamlEnumCon<'a> {
    type Error = DamlLfConvertError;

    fn try_from(enum_con: &DamlEnumConWrapper<'a>) -> DamlLfConvertResult<Self> {
        let tycon = DamlTyConName::try_from(&enum_con.wrap(&enum_con.payload.tycon))?;
        let enum_con = enum_con.payload.enum_con.resolve(enum_con.context.package)?;
        Ok(DamlEnumCon::new(Box::new(tycon), enum_con))
    }
}

impl<'a> TryFrom<&DamlStructConWrapper<'a>> for DamlStructCon<'a> {
    type Error = DamlLfConvertError;

    fn try_from(struct_con: &DamlStructConWrapper<'a>) -> DamlLfConvertResult<Self> {
        let fields = struct_con
            .payload
            .fields
            .iter()
            .map(|field| DamlFieldWithExpr::try_from(&struct_con.wrap(field)))
            .collect::<DamlLfConvertResult<_>>()?;
        Ok(DamlStructCon::new(fields))
    }
}

impl<'a> TryFrom<&DamlStructProjWrapper<'a>> for DamlStructProj<'a> {
    type Error = DamlLfConvertError;

    fn try_from(struct_proj: &DamlStructProjWrapper<'a>) -> DamlLfConvertResult<Self> {
        let struct_expr = DamlExpr::try_from(&struct_proj.wrap(struct_proj.payload.struct_expr.as_ref()))?;
        let field = struct_proj.payload.field.resolve(struct_proj.context.package)?;
        Ok(DamlStructProj::new(Box::new(struct_expr), field))
    }
}

impl<'a> TryFrom<&DamlStructUpdWrapper<'a>> for DamlStructUpd<'a> {
    type Error = DamlLfConvertError;

    fn try_from(struct_upd: &DamlStructUpdWrapper<'a>) -> DamlLfConvertResult<Self> {
        let struct_expr = DamlExpr::try_from(&struct_upd.wrap(struct_upd.payload.struct_expr.as_ref()))?;
        let update = DamlExpr::try_from(&struct_upd.wrap(struct_upd.payload.update.as_ref()))?;
        let field = struct_upd.payload.field.resolve(struct_upd.context.package)?;
        Ok(DamlStructUpd::new(Box::new(struct_expr), Box::new(update), field))
    }
}

impl<'a> TryFrom<&DamlAppWrapper<'a>> for DamlApp<'a> {
    type Error = DamlLfConvertError;

    fn try_from(app: &DamlAppWrapper<'a>) -> DamlLfConvertResult<Self> {
        let fun = DamlExpr::try_from(&app.wrap(app.payload.fun.as_ref()))?;
        let args = app
            .payload
            .args
            .iter()
            .map(|arg| DamlExpr::try_from(&app.wrap(arg)))
            .collect::<DamlLfConvertResult<_>>()?;
        Ok(DamlApp::new(Box::new(fun), args))
    }
}

impl<'a> TryFrom<&DamlTyAppWrapper<'a>> for DamlTyApp<'a> {
    type Error = DamlLfConvertError;

    fn try_from(ty_app: &DamlTyAppWrapper<'a>) -> DamlLfConvertResult<Self> {
        let expr = DamlExpr::try_from(&ty_app.wrap(ty_app.payload.expr.as_ref()))?;
        let types = ty_app
            .payload
            .types
            .iter()
            .map(|ty| DamlType::try_from(&ty_app.wrap(ty)))
            .collect::<DamlLfConvertResult<_>>()?;
        Ok(DamlTyApp::new(Box::new(expr), types))
    }
}

impl<'a> TryFrom<&DamlAbsWrapper<'a>> for DamlAbs<'a> {
    type Error = DamlLfConvertError;

    fn try_from(abs: &DamlAbsWrapper<'a>) -> DamlLfConvertResult<Self> {
        let params = abs
            .payload
            .params
            .iter()
            .map(|var| DamlVarWithType::try_from(&abs.wrap(var)))
            .collect::<DamlLfConvertResult<_>>()?;
        let body = DamlExpr::try_from(&abs.wrap(abs.payload.body.as_ref()))?;
        Ok(DamlAbs::new(params, Box::new(body)))
    }
}

impl<'a> TryFrom<&DamlVarWithTypeWrapper<'a>> for DamlVarWithType<'a> {
    type Error = DamlLfConvertError;

    fn try_from(var_with_type: &DamlVarWithTypeWrapper<'a>) -> DamlLfConvertResult<Self> {
        let ty = DamlType::try_from(&var_with_type.wrap(&var_with_type.payload.ty))?;
        let var = var_with_type.payload.var.resolve(var_with_type.context.package)?;
        Ok(DamlVarWithType::new(ty, var))
    }
}

impl<'a> TryFrom<&DamlTyAbsWrapper<'a>> for DamlTyAbs<'a> {
    type Error = DamlLfConvertError;

    fn try_from(ty_abs: &DamlTyAbsWrapper<'a>) -> DamlLfConvertResult<Self> {
        let params = ty_abs
            .payload
            .params
            .iter()
            .map(|ty_var| DamlTypeVarWithKind::try_from(&ty_abs.wrap(ty_var)))
            .collect::<DamlLfConvertResult<_>>()?;
        let body = DamlExpr::try_from(&ty_abs.wrap(ty_abs.payload.body.as_ref()))?;
        Ok(DamlTyAbs::new(params, Box::new(body)))
    }
}

impl<'a> TryFrom<&DamlCaseWrapper<'a>> for DamlCase<'a> {
    type Error = DamlLfConvertError;

    fn try_from(case: &DamlCaseWrapper<'a>) -> DamlLfConvertResult<Self> {
        let scrut = DamlExpr::try_from(&case.wrap(case.payload.scrut.as_ref()))?;
        let alts = case
            .payload
            .alts
            .iter()
            .map(|alt| DamlCaseAlt::try_from(&case.wrap(alt)))
            .collect::<DamlLfConvertResult<Vec<_>>>()?;
        Ok(DamlCase::new(Box::new(scrut), alts))
    }
}

impl<'a> TryFrom<&DamlCaseAltWrapper<'a>> for DamlCaseAlt<'a> {
    type Error = DamlLfConvertError;

    fn try_from(case_alt: &DamlCaseAltWrapper<'a>) -> DamlLfConvertResult<Self> {
        let body = DamlExpr::try_from(&case_alt.wrap(&case_alt.payload.body))?;
        let sum = DamlCaseAltSum::try_from(&case_alt.wrap(&case_alt.payload.sum))?;
        Ok(DamlCaseAlt::new(body, sum))
    }
}

impl<'a> TryFrom<&DamlCaseAltSumWrapper<'a>> for DamlCaseAltSum<'a> {
    type Error = DamlLfConvertError;

    fn try_from(sum: &DamlCaseAltSumWrapper<'a>) -> DamlLfConvertResult<Self> {
        Ok(match sum.payload {
            DamlCaseAltSumPayload::Default => DamlCaseAltSum::Default,
            DamlCaseAltSumPayload::Variant(variant) =>
                DamlCaseAltSum::Variant(DamlCaseAltVariant::try_from(&sum.wrap(variant))?),
            DamlCaseAltSumPayload::PrimCon(prim_con) => DamlCaseAltSum::PrimCon(DamlPrimCon::try_from(prim_con)?),
            DamlCaseAltSumPayload::Nil => DamlCaseAltSum::Nil,
            DamlCaseAltSumPayload::Cons(cons) => DamlCaseAltSum::Cons(DamlCaseAltCons::try_from(&sum.wrap(cons))?),
            DamlCaseAltSumPayload::OptionalNone => DamlCaseAltSum::OptionalNone,
            DamlCaseAltSumPayload::OptionalSome(opt_some) =>
                DamlCaseAltSum::OptionalSome(DamlCaseAltOptionalSome::try_from(&sum.wrap(opt_some))?),
            DamlCaseAltSumPayload::Enum(enum_alt) =>
                DamlCaseAltSum::Enum(DamlCaseAltEnum::try_from(&sum.wrap(enum_alt))?),
        })
    }
}

impl<'a> TryFrom<&DamlCaseAltVariantWrapper<'a>> for DamlCaseAltVariant<'a> {
    type Error = DamlLfConvertError;

    fn try_from(variant_alt: &DamlCaseAltVariantWrapper<'a>) -> DamlLfConvertResult<Self> {
        let con = DamlTyConName::try_from(&variant_alt.wrap(&variant_alt.payload.con))?;
        let variant = variant_alt.payload.variant.resolve(variant_alt.context.package)?;
        let binder = variant_alt.payload.binder.resolve(variant_alt.context.package)?;
        Ok(DamlCaseAltVariant::new(con, variant, binder))
    }
}

impl<'a> TryFrom<&DamlCaseAltConsWrapper<'a>> for DamlCaseAltCons<'a> {
    type Error = DamlLfConvertError;

    fn try_from(cons_alt: &DamlCaseAltConsWrapper<'a>) -> DamlLfConvertResult<Self> {
        let var_head = cons_alt.payload.var_head.resolve(cons_alt.context.package)?;
        let var_tail = cons_alt.payload.var_tail.resolve(cons_alt.context.package)?;
        Ok(DamlCaseAltCons::new(var_head, var_tail))
    }
}

impl<'a> TryFrom<&DamlCaseAltOptionalSomeWrapper<'a>> for DamlCaseAltOptionalSome<'a> {
    type Error = DamlLfConvertError;

    fn try_from(opt_some_alt: &DamlCaseAltOptionalSomeWrapper<'a>) -> DamlLfConvertResult<Self> {
        let var_body = opt_some_alt.payload.var_body.resolve(opt_some_alt.context.package)?;
        Ok(DamlCaseAltOptionalSome::new(var_body))
    }
}

impl<'a> TryFrom<&DamlCaseAltEnumWrapper<'a>> for DamlCaseAltEnum<'a> {
    type Error = DamlLfConvertError;

    fn try_from(enum_alt: &DamlCaseAltEnumWrapper<'a>) -> DamlLfConvertResult<Self> {
        let con = DamlTyConName::try_from(&enum_alt.wrap(&enum_alt.payload.con))?;
        let constructor = enum_alt.payload.constructor.resolve(enum_alt.context.package)?;
        Ok(DamlCaseAltEnum::new(con, constructor))
    }
}

impl<'a> TryFrom<&DamlBlockWrapper<'a>> for DamlBlock<'a> {
    type Error = DamlLfConvertError;

    fn try_from(block: &DamlBlockWrapper<'a>) -> DamlLfConvertResult<Self> {
        let bindings = block
            .payload
            .bindings
            .iter()
            .map(|binding| DamlBinding::try_from(&block.wrap(binding)))
            .collect::<DamlLfConvertResult<_>>()?;
        let body = DamlExpr::try_from(&block.wrap(block.payload.body.as_ref()))?;
        Ok(DamlBlock::new(bindings, Box::new(body)))
    }
}

impl<'a> TryFrom<&DamlBindingWrapper<'a>> for DamlBinding<'a> {
    type Error = DamlLfConvertError;

    fn try_from(binding: &DamlBindingWrapper<'a>) -> DamlLfConvertResult<Self> {
        let binder = DamlVarWithType::try_from(&binding.wrap(&binding.payload.binder))?;
        let bound = DamlExpr::try_from(&binding.wrap(&binding.payload.bound))?;
        Ok(DamlBinding::new(binder, bound))
    }
}

impl<'a> TryFrom<&DamlConsWrapper<'a>> for DamlCons<'a> {
    type Error = DamlLfConvertError;

    fn try_from(cons: &DamlConsWrapper<'a>) -> DamlLfConvertResult<Self> {
        let ty = DamlType::try_from(&cons.wrap(&cons.payload.ty))?;
        let front = cons
            .payload
            .front
            .iter()
            .map(|expr| DamlExpr::try_from(&cons.wrap(expr)))
            .collect::<DamlLfConvertResult<_>>()?;
        let tail = DamlExpr::try_from(&cons.wrap(cons.payload.tail.as_ref()))?;
        Ok(DamlCons::new(ty, front, Box::new(tail)))
    }
}

impl<'a> TryFrom<&DamlOptionalSomeWrapper<'a>> for DamlOptionalSome<'a> {
    type Error = DamlLfConvertError;

    fn try_from(opt_some: &DamlOptionalSomeWrapper<'a>) -> DamlLfConvertResult<Self> {
        let ty = DamlType::try_from(&opt_some.wrap(&opt_some.payload.ty))?;
        let body = DamlExpr::try_from(&opt_some.wrap(opt_some.payload.body.as_ref()))?;
        Ok(DamlOptionalSome::new(ty, Box::new(body)))
    }
}

impl<'a> TryFrom<&DamlToAnyWrapper<'a>> for DamlToAny<'a> {
    type Error = DamlLfConvertError;

    fn try_from(to_any: &DamlToAnyWrapper<'a>) -> DamlLfConvertResult<Self> {
        let ty = DamlType::try_from(&to_any.wrap(&to_any.payload.ty))?;
        let expr = DamlExpr::try_from(&to_any.wrap(to_any.payload.expr.as_ref()))?;
        Ok(DamlToAny::new(ty, Box::new(expr)))
    }
}

impl<'a> TryFrom<&DamlFromAnyWrapper<'a>> for DamlFromAny<'a> {
    type Error = DamlLfConvertError;

    fn try_from(from_any: &DamlFromAnyWrapper<'a>) -> DamlLfConvertResult<Self> {
        let ty = DamlType::try_from(&from_any.wrap(&from_any.payload.ty))?;
        let expr = DamlExpr::try_from(&from_any.wrap(from_any.payload.expr.as_ref()))?;
        Ok(DamlFromAny::new(ty, Box::new(expr)))
    }
}

impl<'a> TryFrom<&DamlUpdateWrapper<'a>> for DamlUpdate<'a> {
    type Error = DamlLfConvertError;

    fn try_from(update: &DamlUpdateWrapper<'a>) -> DamlLfConvertResult<Self> {
        Ok(match update.payload {
            DamlUpdatePayload::Pure(pure) => DamlUpdate::Pure(DamlPure::try_from(&update.wrap(pure))?),
            DamlUpdatePayload::Block(block) => DamlUpdate::Block(DamlBlock::try_from(&update.wrap(block))?),
            DamlUpdatePayload::Create(create) => DamlUpdate::Create(DamlCreate::try_from(&update.wrap(create))?),
            DamlUpdatePayload::Exercise(exercise) =>
                DamlUpdate::Exercise(DamlExercise::try_from(&update.wrap(exercise))?),
            DamlUpdatePayload::ExerciseByKey(exercise_by_key) =>
                DamlUpdate::ExerciseByKey(DamlExerciseByKey::try_from(&update.wrap(exercise_by_key))?),
            DamlUpdatePayload::Fetch(fetch) => DamlUpdate::Fetch(DamlFetch::try_from(&update.wrap(fetch))?),
            DamlUpdatePayload::GetTime => DamlUpdate::GetTime,
            DamlUpdatePayload::LookupByKey(retrieve_by_key) =>
                DamlUpdate::LookupByKey(DamlRetrieveByKey::try_from(&update.wrap(retrieve_by_key))?),
            DamlUpdatePayload::FetchByKey(retrieve_by_key) =>
                DamlUpdate::FetchByKey(DamlRetrieveByKey::try_from(&update.wrap(retrieve_by_key))?),
            DamlUpdatePayload::EmbedExpr(embed_expr) =>
                DamlUpdate::EmbedExpr(DamlUpdateEmbedExpr::try_from(&update.wrap(embed_expr))?),
            DamlUpdatePayload::TryCatch(try_catch) =>
                DamlUpdate::TryCatch(DamlTryCatch::try_from(&update.wrap(try_catch))?),
        })
    }
}

impl<'a> TryFrom<&DamlPureWrapper<'a>> for DamlPure<'a> {
    type Error = DamlLfConvertError;

    fn try_from(pure: &DamlPureWrapper<'a>) -> DamlLfConvertResult<Self> {
        let ty = DamlType::try_from(&pure.wrap(&pure.payload.ty))?;
        let expr = DamlExpr::try_from(&pure.wrap(pure.payload.expr.as_ref()))?;
        Ok(DamlPure::new(ty, Box::new(expr)))
    }
}

impl<'a> TryFrom<&DamlCreateWrapper<'a>> for DamlCreate<'a> {
    type Error = DamlLfConvertError;

    fn try_from(create: &DamlCreateWrapper<'a>) -> DamlLfConvertResult<Self> {
        let template = DamlTyConName::try_from(&create.wrap(&create.payload.template))?;
        let expr = DamlExpr::try_from(&create.wrap(create.payload.expr.as_ref()))?;
        Ok(DamlCreate::new(Box::new(template), Box::new(expr)))
    }
}

impl<'a> TryFrom<&DamlExerciseWrapper<'a>> for DamlExercise<'a> {
    type Error = DamlLfConvertError;

    fn try_from(exercise: &DamlExerciseWrapper<'a>) -> DamlLfConvertResult<Self> {
        let template = DamlTyConName::try_from(&exercise.wrap(&exercise.payload.template))?;
        let cid = DamlExpr::try_from(&exercise.wrap(exercise.payload.cid.as_ref()))?;
        let arg = DamlExpr::try_from(&exercise.wrap(exercise.payload.arg.as_ref()))?;
        let choice = exercise.payload.choice.resolve(exercise.context.package)?;
        Ok(DamlExercise::new(Box::new(template), Box::new(cid), Box::new(arg), choice))
    }
}

impl<'a> TryFrom<&DamlExerciseByKeyWrapper<'a>> for DamlExerciseByKey<'a> {
    type Error = DamlLfConvertError;

    fn try_from(exercise_by_key: &DamlExerciseByKeyWrapper<'a>) -> DamlLfConvertResult<Self> {
        let template = DamlTyConName::try_from(&exercise_by_key.wrap(&exercise_by_key.payload.template))?;
        let choice = exercise_by_key.payload.choice.resolve(exercise_by_key.context.package)?;
        let key = DamlExpr::try_from(&exercise_by_key.wrap(exercise_by_key.payload.key.as_ref()))?;
        let arg = DamlExpr::try_from(&exercise_by_key.wrap(exercise_by_key.payload.arg.as_ref()))?;
        Ok(DamlExerciseByKey::new(Box::new(template), choice, Box::new(key), Box::new(arg)))
    }
}

impl<'a> TryFrom<&DamlFetchWrapper<'a>> for DamlFetch<'a> {
    type Error = DamlLfConvertError;

    fn try_from(fetch: &DamlFetchWrapper<'a>) -> DamlLfConvertResult<Self> {
        let template = DamlTyConName::try_from(&fetch.wrap(&fetch.payload.template))?;
        let cid = DamlExpr::try_from(&fetch.wrap(fetch.payload.cid.as_ref()))?;
        Ok(DamlFetch::new(Box::new(template), Box::new(cid)))
    }
}

impl<'a> TryFrom<&DamlRetrieveByKeyWrapper<'a>> for DamlRetrieveByKey<'a> {
    type Error = DamlLfConvertError;

    fn try_from(retrieve_by_key: &DamlRetrieveByKeyWrapper<'a>) -> DamlLfConvertResult<Self> {
        let template = DamlTyConName::try_from(&retrieve_by_key.wrap(&retrieve_by_key.payload.template))?;
        let key = DamlExpr::try_from(&retrieve_by_key.wrap(retrieve_by_key.payload.key.as_ref()))?;
        Ok(DamlRetrieveByKey::new(Box::new(template), Box::new(key)))
    }
}

impl<'a> TryFrom<&DamlUpdateEmbedExprWrapper<'a>> for DamlUpdateEmbedExpr<'a> {
    type Error = DamlLfConvertError;

    fn try_from(embed_expr: &DamlUpdateEmbedExprWrapper<'a>) -> DamlLfConvertResult<Self> {
        let ty = DamlType::try_from(&embed_expr.wrap(&embed_expr.payload.ty))?;
        let body = DamlExpr::try_from(&embed_expr.wrap(embed_expr.payload.body.as_ref()))?;
        Ok(DamlUpdateEmbedExpr::new(ty, Box::new(body)))
    }
}

impl<'a> TryFrom<&DamlScenarioWrapper<'a>> for DamlScenario<'a> {
    type Error = DamlLfConvertError;

    fn try_from(update: &DamlScenarioWrapper<'a>) -> DamlLfConvertResult<Self> {
        Ok(match update.payload {
            DamlScenarioPayload::Pure(pure) => DamlScenario::Pure(DamlPure::try_from(&update.wrap(pure))?),
            DamlScenarioPayload::Block(block) => DamlScenario::Block(DamlBlock::try_from(&update.wrap(block))?),
            DamlScenarioPayload::Commit(commit) => DamlScenario::Commit(DamlCommit::try_from(&update.wrap(commit))?),
            DamlScenarioPayload::MustFailAt(commit) =>
                DamlScenario::MustFailAt(DamlCommit::try_from(&update.wrap(commit))?),
            DamlScenarioPayload::Pass(expr) =>
                DamlScenario::Pass(Box::new(DamlExpr::try_from(&update.wrap(expr.as_ref()))?)),
            DamlScenarioPayload::GetTime => DamlScenario::GetTime,
            DamlScenarioPayload::GetParty(expr) =>
                DamlScenario::GetParty(Box::new(DamlExpr::try_from(&update.wrap(expr.as_ref()))?)),
            DamlScenarioPayload::EmbedExpr(embed_expr) =>
                DamlScenario::EmbedExpr(DamlScenarioEmbedExpr::try_from(&update.wrap(embed_expr))?),
        })
    }
}

impl<'a> TryFrom<&DamlCommitWrapper<'a>> for DamlCommit<'a> {
    type Error = DamlLfConvertError;

    fn try_from(commit: &DamlCommitWrapper<'a>) -> DamlLfConvertResult<Self> {
        let expr = DamlExpr::try_from(&commit.wrap(commit.payload.expr.as_ref()))?;
        let party = DamlExpr::try_from(&commit.wrap(commit.payload.party.as_ref()))?;
        let ret_type = DamlType::try_from(&commit.wrap(&commit.payload.ret_type))?;
        Ok(DamlCommit::new(Box::new(party), Box::new(expr), ret_type))
    }
}

impl<'a> TryFrom<&DamlScenarioEmbedExprWrapper<'a>> for DamlScenarioEmbedExpr<'a> {
    type Error = DamlLfConvertError;

    fn try_from(embed_expr: &DamlScenarioEmbedExprWrapper<'a>) -> DamlLfConvertResult<Self> {
        let ty = DamlType::try_from(&embed_expr.wrap(&embed_expr.payload.ty))?;
        let body = DamlExpr::try_from(&embed_expr.wrap(embed_expr.payload.body.as_ref()))?;
        Ok(DamlScenarioEmbedExpr::new(ty, Box::new(body)))
    }
}

impl<'a> TryFrom<&DamlValueNameWrapper<'a>> for DamlValueName<'a> {
    type Error = DamlLfConvertError;

    fn try_from(value_name: &DamlValueNameWrapper<'a>) -> DamlLfConvertResult<Self> {
        let source_resolver = value_name.context.package;
        let source_package_id = Cow::from(value_name.context.package.package_id);
        let source_package_name = Cow::from(value_name.context.package.name.as_str());
        let source_module_path = value_name.context.module.path.resolve(source_resolver)?;
        let target_package_id = value_name.payload.package_ref.resolve(source_resolver)?;
        let target_package: &DamlPackagePayload<'_> = value_name
            .context
            .archive
            .package_by_id(&target_package_id)
            .ok_or_else(|| DamlLfConvertError::UnknownPackage(target_package_id.to_string()))?;
        let target_package_name = Cow::from(target_package.name.as_str());
        let target_module_path = value_name.payload.module_path.resolve(source_resolver)?;
        let data_name = value_name.payload.name.resolve_last(source_resolver)?;
        if target_package_name == source_package_name && target_module_path == source_module_path {
            Ok(DamlValueName::Local(DamlLocalValueName::new(
                data_name,
                target_package_id,
                target_package_name,
                target_module_path,
            )))
        } else {
            Ok(DamlValueName::NonLocal(DamlNonLocalValueName::new(
                data_name,
                source_package_id,
                source_package_name,
                source_module_path,
                target_package_id,
                target_package_name,
                target_module_path,
            )))
        }
    }
}

impl<'a> TryFrom<&DamlToAnyExceptionWrapper<'a>> for DamlToAnyException<'a> {
    type Error = DamlLfConvertError;

    fn try_from(to_any_exception: &DamlToAnyExceptionWrapper<'a>) -> DamlLfConvertResult<Self> {
        let ty = DamlType::try_from(&to_any_exception.wrap(&to_any_exception.payload.ty))?;
        let expr = DamlExpr::try_from(&to_any_exception.wrap(to_any_exception.payload.expr.as_ref()))?;
        Ok(DamlToAnyException::new(ty, Box::new(expr)))
    }
}

impl<'a> TryFrom<&DamlFromAnyExceptionWrapper<'a>> for DamlFromAnyException<'a> {
    type Error = DamlLfConvertError;

    fn try_from(from_any_exception: &DamlFromAnyExceptionWrapper<'a>) -> DamlLfConvertResult<Self> {
        let ty = DamlType::try_from(&from_any_exception.wrap(&from_any_exception.payload.ty))?;
        let expr = DamlExpr::try_from(&from_any_exception.wrap(from_any_exception.payload.expr.as_ref()))?;
        Ok(DamlFromAnyException::new(ty, Box::new(expr)))
    }
}

impl<'a> TryFrom<&DamlThrowWrapper<'a>> for DamlThrow<'a> {
    type Error = DamlLfConvertError;

    fn try_from(throw: &DamlThrowWrapper<'a>) -> DamlLfConvertResult<Self> {
        let return_type = DamlType::try_from(&throw.wrap(&throw.payload.return_type))?;
        let exception_type = DamlType::try_from(&throw.wrap(&throw.payload.exception_type))?;
        let exception_expr = DamlExpr::try_from(&throw.wrap(throw.payload.exception_expr.as_ref()))?;
        Ok(DamlThrow::new(return_type, exception_type, Box::new(exception_expr)))
    }
}

impl<'a> TryFrom<&DamlTryCatchWrapper<'a>> for DamlTryCatch<'a> {
    type Error = DamlLfConvertError;

    fn try_from(try_catch: &DamlTryCatchWrapper<'a>) -> DamlLfConvertResult<Self> {
        let return_type = DamlType::try_from(&try_catch.wrap(&try_catch.payload.return_type))?;
        let try_expr = DamlExpr::try_from(&try_catch.wrap(try_catch.payload.try_expr.as_ref()))?;
        let var = try_catch.payload.var.resolve(try_catch.context.package)?;
        let catch_expr = DamlExpr::try_from(&try_catch.wrap(try_catch.payload.catch_expr.as_ref()))?;
        Ok(DamlTryCatch::new(return_type, Box::new(try_expr), var, Box::new(catch_expr)))
    }
}
