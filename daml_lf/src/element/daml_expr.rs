use crate::element::{
    DamlElementVisitor, DamlTyCon, DamlTyConName, DamlType, DamlTypeVarWithKind, DamlVisitableElement,
};
use crate::owned::ToStatic;
use serde::Serialize;
use std::borrow::Cow;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Clone)]
pub enum DamlExpr<'a> {
    Var(Cow<'a, str>),
    Val(Box<DamlValueName<'a>>),
    Builtin(DamlBuiltinFunction),
    PrimCon(DamlPrimCon),
    PrimLit(DamlPrimLit<'a>),
    RecCon(DamlRecCon<'a>),
    RecProj(DamlRecProj<'a>),
    RecUpd(DamlRecUpd<'a>),
    VariantCon(DamlVariantCon<'a>),
    EnumCon(DamlEnumCon<'a>),
    StructCon(DamlStructCon<'a>),
    StructProj(DamlStructProj<'a>),
    StructUpd(DamlStructUpd<'a>),
    App(DamlApp<'a>),
    TyApp(DamlTyApp<'a>),
    Abs(DamlAbs<'a>),
    TyAbs(DamlTyAbs<'a>),
    Case(DamlCase<'a>),
    Let(DamlBlock<'a>),
    Nil(DamlType<'a>),
    Cons(DamlCons<'a>),
    Update(DamlUpdate<'a>),
    Scenario(DamlScenario<'a>),
    OptionalNone(DamlType<'a>),
    OptionalSome(DamlOptionalSome<'a>),
    ToAny(DamlToAny<'a>),
    FromAny(DamlFromAny<'a>),
    TypeRep(DamlType<'a>),
}

impl<'a> DamlVisitableElement<'a> for DamlExpr<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_expr(self);
        match self {
            DamlExpr::Val(val) => val.accept(visitor),
            DamlExpr::Builtin(builtin) => builtin.accept(visitor),
            DamlExpr::PrimCon(prim_con) => prim_con.accept(visitor),
            DamlExpr::PrimLit(prim_lit) => prim_lit.accept(visitor),
            DamlExpr::RecCon(rec_con) => rec_con.accept(visitor),
            DamlExpr::RecProj(rec_proj) => rec_proj.accept(visitor),
            DamlExpr::RecUpd(rec_upd) => rec_upd.accept(visitor),
            DamlExpr::VariantCon(variant_con) => variant_con.accept(visitor),
            DamlExpr::EnumCon(enum_con) => enum_con.accept(visitor),
            DamlExpr::StructCon(struct_con) => struct_con.accept(visitor),
            DamlExpr::StructProj(struct_proj) => struct_proj.accept(visitor),
            DamlExpr::StructUpd(struct_upd) => struct_upd.accept(visitor),
            DamlExpr::App(app) => app.accept(visitor),
            DamlExpr::TyApp(ty_app) => ty_app.accept(visitor),
            DamlExpr::Abs(abs) => abs.accept(visitor),
            DamlExpr::TyAbs(ty_abs) => ty_abs.accept(visitor),
            DamlExpr::Case(case) => case.accept(visitor),
            DamlExpr::Let(block) => block.accept(visitor),
            DamlExpr::Cons(cons) => cons.accept(visitor),
            DamlExpr::Update(update) => update.accept(visitor),
            DamlExpr::Scenario(scenario) => scenario.accept(visitor),
            DamlExpr::OptionalSome(opt_some) => opt_some.accept(visitor),
            DamlExpr::ToAny(to_any) => to_any.accept(visitor),
            DamlExpr::FromAny(from_any) => from_any.accept(visitor),
            DamlExpr::TypeRep(ty) | DamlExpr::OptionalNone(ty) | DamlExpr::Nil(ty) => ty.accept(visitor),
            DamlExpr::Var(_) => {},
        }
        visitor.post_visit_expr(self);
    }
}

impl ToStatic for DamlExpr<'_> {
    type Static = DamlExpr<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            DamlExpr::Var(var) => DamlExpr::Var(var.to_static()),
            DamlExpr::Val(val) => DamlExpr::Val(Box::new(val.to_static())),
            DamlExpr::Builtin(builtin) => DamlExpr::Builtin(builtin.clone()),
            DamlExpr::PrimCon(prim_con) => DamlExpr::PrimCon(*prim_con),
            DamlExpr::PrimLit(prim_lit) => DamlExpr::PrimLit(prim_lit.to_static()),
            DamlExpr::RecCon(rec_con) => DamlExpr::RecCon(rec_con.to_static()),
            DamlExpr::RecProj(rec_proj) => DamlExpr::RecProj(rec_proj.to_static()),
            DamlExpr::RecUpd(rec_upd) => DamlExpr::RecUpd(rec_upd.to_static()),
            DamlExpr::VariantCon(variant_con) => DamlExpr::VariantCon(variant_con.to_static()),
            DamlExpr::EnumCon(enum_con) => DamlExpr::EnumCon(enum_con.to_static()),
            DamlExpr::StructCon(struct_con) => DamlExpr::StructCon(struct_con.to_static()),
            DamlExpr::StructProj(struct_proj) => DamlExpr::StructProj(struct_proj.to_static()),
            DamlExpr::StructUpd(struct_upd) => DamlExpr::StructUpd(struct_upd.to_static()),
            DamlExpr::App(app) => DamlExpr::App(app.to_static()),
            DamlExpr::TyApp(ty_app) => DamlExpr::TyApp(ty_app.to_static()),
            DamlExpr::Abs(abs) => DamlExpr::Abs(abs.to_static()),
            DamlExpr::TyAbs(ty_abs) => DamlExpr::TyAbs(ty_abs.to_static()),
            DamlExpr::Case(case) => DamlExpr::Case(case.to_static()),
            DamlExpr::Let(block) => DamlExpr::Let(block.to_static()),
            DamlExpr::Nil(ty) => DamlExpr::Nil(ty.to_static()),
            DamlExpr::Cons(cons) => DamlExpr::Cons(cons.to_static()),
            DamlExpr::Update(update) => DamlExpr::Update(update.to_static()),
            DamlExpr::Scenario(scenario) => DamlExpr::Scenario(scenario.to_static()),
            DamlExpr::OptionalNone(ty) => DamlExpr::OptionalNone(ty.to_static()),
            DamlExpr::OptionalSome(opt_some) => DamlExpr::OptionalSome(opt_some.to_static()),
            DamlExpr::ToAny(to_any) => DamlExpr::ToAny(to_any.to_static()),
            DamlExpr::FromAny(from_any) => DamlExpr::FromAny(from_any.to_static()),
            DamlExpr::TypeRep(ty) => DamlExpr::TypeRep(ty.to_static()),
        }
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub enum DamlValueName<'a> {
    Local(DamlLocalValueName<'a>),
    NonLocal(DamlNonLocalValueName<'a>),
}

impl DamlValueName<'_> {
    pub fn name(&self) -> &str {
        match self {
            DamlValueName::Local(local) => local.name(),
            DamlValueName::NonLocal(non_local) => non_local.name(),
        }
    }

    pub fn package_id(&self) -> &str {
        match self {
            DamlValueName::Local(local) => local.package_id(),
            DamlValueName::NonLocal(non_local) => non_local.target_package_id(),
        }
    }

    pub fn package_name(&self) -> &str {
        match self {
            DamlValueName::Local(local) => local.package_name(),
            DamlValueName::NonLocal(non_local) => non_local.target_package_name(),
        }
    }

    pub fn module_path(&self) -> impl Iterator<Item = &str> {
        match self {
            DamlValueName::Local(local) => local.module_path.iter().map(AsRef::as_ref),
            DamlValueName::NonLocal(non_local) => non_local.target_module_path.iter().map(AsRef::as_ref),
        }
    }

    /// Extract the package id, module path and name.
    #[doc(hidden)]
    pub(crate) fn reference_parts(&self) -> (&str, &[Cow<'_, str>], &str) {
        match self {
            DamlValueName::Local(local) => (&local.package_id, &local.module_path, &local.name),
            DamlValueName::NonLocal(non_local) =>
                (&non_local.target_package_id, &non_local.target_module_path, &non_local.name),
        }
    }
}

impl Display for DamlValueName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DamlValueName::Local(local) =>
                write!(f, "{}:{}:{}", local.package_name, &local.module_path.join("."), local.name),
            DamlValueName::NonLocal(non_local) => write!(
                f,
                "{}:{}:{}",
                non_local.target_package_name,
                &non_local.target_module_path.join("."),
                non_local.name
            ),
        }
    }
}

impl<'a> DamlVisitableElement<'a> for DamlValueName<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_value_name(self);
        match self {
            DamlValueName::Local(local) => local.accept(visitor),
            DamlValueName::NonLocal(non_local) => non_local.accept(visitor),
        }
        visitor.post_visit_value_name(self);
    }
}

impl ToStatic for DamlValueName<'_> {
    type Static = DamlValueName<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            DamlValueName::Local(local) => DamlValueName::Local(local.to_static()),
            DamlValueName::NonLocal(non_local) => DamlValueName::NonLocal(non_local.to_static()),
        }
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlLocalValueName<'a> {
    pub name: Cow<'a, str>,
    pub package_id: Cow<'a, str>,
    pub package_name: Cow<'a, str>,
    pub module_path: Vec<Cow<'a, str>>,
}

impl<'a> DamlLocalValueName<'a> {
    pub fn new(
        name: Cow<'a, str>,
        package_id: Cow<'a, str>,
        package_name: Cow<'a, str>,
        module_path: Vec<Cow<'a, str>>,
    ) -> Self {
        Self {
            name,
            package_id,
            package_name,
            module_path,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn package_name(&self) -> &str {
        &self.package_name
    }

    pub fn module_path(&self) -> impl Iterator<Item = &str> {
        self.module_path.iter().map(AsRef::as_ref)
    }
}

impl<'a> DamlVisitableElement<'a> for DamlLocalValueName<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_local_value_name(self);
        visitor.post_visit_local_value_name(self);
    }
}

impl ToStatic for DamlLocalValueName<'_> {
    type Static = DamlLocalValueName<'static>;

    fn to_static(&self) -> Self::Static {
        DamlLocalValueName::new(
            self.name.to_static(),
            self.package_id.to_static(),
            self.package_name.to_static(),
            self.module_path.iter().map(ToStatic::to_static).collect(),
        )
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlNonLocalValueName<'a> {
    name: Cow<'a, str>,
    source_package_id: Cow<'a, str>,
    source_package_name: Cow<'a, str>,
    source_module_path: Vec<Cow<'a, str>>,
    target_package_id: Cow<'a, str>,
    target_package_name: Cow<'a, str>,
    target_module_path: Vec<Cow<'a, str>>,
}

impl<'a> DamlNonLocalValueName<'a> {
    pub fn new(
        name: Cow<'a, str>,
        source_package_id: Cow<'a, str>,
        source_package_name: Cow<'a, str>,
        source_module_path: Vec<Cow<'a, str>>,
        target_package_id: Cow<'a, str>,
        target_package_name: Cow<'a, str>,
        target_module_path: Vec<Cow<'a, str>>,
    ) -> Self {
        Self {
            name,
            source_package_id,
            source_package_name,
            source_module_path,
            target_package_id,
            target_package_name,
            target_module_path,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_package_id(&self) -> &str {
        &self.source_package_id
    }

    pub fn source_package_name(&self) -> &str {
        &self.source_package_name
    }

    pub fn source_module_path(&self) -> impl Iterator<Item = &str> {
        self.source_module_path.iter().map(AsRef::as_ref)
    }

    pub fn target_package_id(&self) -> &str {
        &self.target_package_id
    }

    pub fn target_package_name(&self) -> &str {
        &self.target_package_name
    }

    pub fn target_module_path(&self) -> impl Iterator<Item = &str> {
        self.target_module_path.iter().map(AsRef::as_ref)
    }
}

impl<'a> DamlVisitableElement<'a> for DamlNonLocalValueName<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_non_local_value_name(self);
        visitor.post_visit_non_local_value_name(self);
    }
}

impl ToStatic for DamlNonLocalValueName<'_> {
    type Static = DamlNonLocalValueName<'static>;

    fn to_static(&self) -> Self::Static {
        DamlNonLocalValueName::new(
            self.name.to_static(),
            self.source_package_id.to_static(),
            self.source_package_name.to_static(),
            self.source_module_path.iter().map(ToStatic::to_static).collect(),
            self.target_package_id.to_static(),
            self.target_package_name.to_static(),
            self.target_module_path.iter().map(ToStatic::to_static).collect(),
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum DamlBuiltinFunction {
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
    ScaleBignumeric,
    PrecisionBignumeric,
    AddBignumeric,
    SubBignumeric,
    MulBignumeric,
    DivBignumeric,
    ShiftBignumeric,
    ToNumericBignumeric,
    ShiftRightBignumeric,
    ToBignumericNumeric,
    ToTextBignumeric,
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
}

impl<'a> DamlVisitableElement<'a> for DamlBuiltinFunction {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_builtin_function(self);
        visitor.post_visit_builtin_function(self);
    }
}

#[derive(Debug, Serialize, Copy, Clone)]
pub enum DamlPrimCon {
    Unit,
    False,
    True,
}

impl<'a> DamlVisitableElement<'a> for DamlPrimCon {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_prim_con(*self);
        visitor.post_visit_prim_con(*self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum DamlPrimLit<'a> {
    /// Represents a standard signed 64-bit integer (integer between −2⁶³ to 2⁶³−1).
    Int64(i64),
    /// Represents a UTF8 string.
    Text(Cow<'a, str>),
    /// A LitParty represents a party.
    Party(Cow<'a, str>),
    /// Represents the number of day since 1970-01-01 with allowed range from 0001-01-01 to 9999-12-31 and
    /// using a year-month-day format.
    Date(i32),
    /// Represents the number of microseconds since 1970-01-01T00:00:00.000000Z with allowed range
    /// 0001-01-01T00:00:00.000000Z to 9999-12-31T23:59:59.999999Z using a
    /// year-month-day-hour-minute-second-microsecond format.
    Timestamp(i64),
    /// Represents a signed number that can be represented in base-10 without loss of precision with at
    /// most 38 digits (ignoring possible leading 0 and with a scale (the number of significant digits on the right of
    /// the decimal point) between 0 and 37 (bounds inclusive). In the following, we will use scale(LitNumeric) to
    /// denote the scale of the decimal number.
    Numeric(Cow<'a, str>),
    RoundingMode(RoundingMode),
}

impl<'a> DamlVisitableElement<'a> for DamlPrimLit<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_prim_lit(self);
        visitor.post_visit_prim_lit(self);
    }
}

impl ToStatic for DamlPrimLit<'_> {
    type Static = DamlPrimLit<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            DamlPrimLit::Int64(i) => DamlPrimLit::Int64(*i),
            DamlPrimLit::Text(text) => DamlPrimLit::Text(text.to_static()),
            DamlPrimLit::Party(party) => DamlPrimLit::Party(party.to_static()),
            DamlPrimLit::Date(date) => DamlPrimLit::Date(*date),
            DamlPrimLit::Timestamp(timestamp) => DamlPrimLit::Timestamp(*timestamp),
            DamlPrimLit::Numeric(numeric) => DamlPrimLit::Numeric(numeric.to_static()),
            DamlPrimLit::RoundingMode(mode) => DamlPrimLit::RoundingMode(mode.clone()),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum RoundingMode {
    Up,
    Down,
    Ceiling,
    Floor,
    HalfUp,
    HalfDown,
    HalfEven,
    Unnecessary,
}

impl<'a> DamlVisitableElement<'a> for RoundingMode {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_rounding_mode(self);
        visitor.post_visit_rounding_mode(self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlRecCon<'a> {
    tycon: DamlTyCon<'a>,
    fields: Vec<DamlFieldWithExpr<'a>>,
}

impl<'a> DamlRecCon<'a> {
    pub fn new(tycon: DamlTyCon<'a>, fields: Vec<DamlFieldWithExpr<'a>>) -> Self {
        Self {
            tycon,
            fields,
        }
    }

    pub fn tycon(&self) -> &DamlTyCon<'a> {
        &self.tycon
    }

    pub fn fields(&self) -> &[DamlFieldWithExpr<'a>] {
        &self.fields
    }
}

impl<'a> DamlVisitableElement<'a> for DamlRecCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_rec_con(self);
        self.tycon.accept(visitor);
        self.fields.iter().for_each(|field| field.accept(visitor));
        visitor.post_visit_rec_con(self);
    }
}

impl ToStatic for DamlRecCon<'_> {
    type Static = DamlRecCon<'static>;

    fn to_static(&self) -> Self::Static {
        DamlRecCon::new(self.tycon.to_static(), self.fields.iter().map(DamlFieldWithExpr::to_static).collect())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlFieldWithExpr<'a> {
    field: Cow<'a, str>,
    expr: DamlExpr<'a>,
}

impl<'a> DamlFieldWithExpr<'a> {
    pub fn new(field: Cow<'a, str>, expr: DamlExpr<'a>) -> Self {
        Self {
            field,
            expr,
        }
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn expr(&self) -> &DamlExpr<'a> {
        &self.expr
    }
}

impl<'a> DamlVisitableElement<'a> for DamlFieldWithExpr<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_field_with_expr(self);
        self.expr.accept(visitor);
        visitor.post_visit_field_with_expr(self);
    }
}

impl ToStatic for DamlFieldWithExpr<'_> {
    type Static = DamlFieldWithExpr<'static>;

    fn to_static(&self) -> Self::Static {
        DamlFieldWithExpr::new(self.field.to_static(), self.expr.to_static())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlRecProj<'a> {
    tycon: DamlTyCon<'a>,
    record: Box<DamlExpr<'a>>,
    field: Cow<'a, str>,
}

impl<'a> DamlRecProj<'a> {
    pub fn new(tycon: DamlTyCon<'a>, record: Box<DamlExpr<'a>>, field: Cow<'a, str>) -> Self {
        Self {
            tycon,
            record,
            field,
        }
    }

    pub fn tycon(&self) -> &DamlTyCon<'a> {
        &self.tycon
    }

    pub fn record(&self) -> &DamlExpr<'a> {
        self.record.as_ref()
    }

    pub fn field(&self) -> &str {
        &self.field
    }
}

impl<'a> DamlVisitableElement<'a> for DamlRecProj<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_rec_proj(self);
        self.tycon.accept(visitor);
        self.record.accept(visitor);
        visitor.post_visit_rec_proj(self);
    }
}

impl ToStatic for DamlRecProj<'_> {
    type Static = DamlRecProj<'static>;

    fn to_static(&self) -> Self::Static {
        DamlRecProj::new(self.tycon.to_static(), Box::new(self.record.to_static()), self.field.to_static())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlRecUpd<'a> {
    tycon: DamlTyCon<'a>,
    record: Box<DamlExpr<'a>>,
    update: Box<DamlExpr<'a>>,
    field: Cow<'a, str>,
}

impl<'a> DamlRecUpd<'a> {
    pub fn new(
        tycon: DamlTyCon<'a>,
        record: Box<DamlExpr<'a>>,
        update: Box<DamlExpr<'a>>,
        field: Cow<'a, str>,
    ) -> Self {
        Self {
            tycon,
            record,
            update,
            field,
        }
    }

    pub fn tycon(&self) -> &DamlTyCon<'a> {
        &self.tycon
    }

    pub fn record(&self) -> &DamlExpr<'a> {
        self.record.as_ref()
    }

    pub fn update(&self) -> &DamlExpr<'a> {
        self.update.as_ref()
    }

    pub fn field(&self) -> &str {
        &self.field
    }
}

impl<'a> DamlVisitableElement<'a> for DamlRecUpd<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_rec_upd(self);
        self.tycon.accept(visitor);
        self.record.accept(visitor);
        self.update.accept(visitor);
        visitor.post_visit_rec_upd(self);
    }
}

impl ToStatic for DamlRecUpd<'_> {
    type Static = DamlRecUpd<'static>;

    fn to_static(&self) -> Self::Static {
        DamlRecUpd::new(
            self.tycon.to_static(),
            Box::new(self.record.to_static()),
            Box::new(self.update.to_static()),
            self.field.to_static(),
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlVariantCon<'a> {
    tycon: DamlTyCon<'a>,
    variant_arg: Box<DamlExpr<'a>>,
    variant_con: Cow<'a, str>,
}

impl<'a> DamlVariantCon<'a> {
    pub fn new(tycon: DamlTyCon<'a>, variant_arg: Box<DamlExpr<'a>>, variant_con: Cow<'a, str>) -> Self {
        Self {
            tycon,
            variant_arg,
            variant_con,
        }
    }

    pub fn tycon(&self) -> &DamlTyCon<'a> {
        &self.tycon
    }

    pub fn variant_arg(&self) -> &DamlExpr<'a> {
        self.variant_arg.as_ref()
    }

    pub fn variant_con(&self) -> &str {
        &self.variant_con
    }
}

impl<'a> DamlVisitableElement<'a> for DamlVariantCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_variant_con(self);
        self.tycon.accept(visitor);
        self.variant_arg.accept(visitor);
        visitor.post_visit_variant_con(self);
    }
}

impl ToStatic for DamlVariantCon<'_> {
    type Static = DamlVariantCon<'static>;

    fn to_static(&self) -> Self::Static {
        DamlVariantCon::new(
            self.tycon.to_static(),
            Box::new(self.variant_arg.to_static()),
            self.variant_con.to_static(),
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlEnumCon<'a> {
    tycon: Box<DamlTyConName<'a>>,
    enum_con: Cow<'a, str>,
}

impl<'a> DamlEnumCon<'a> {
    pub fn new(tycon: DamlTyConName<'a>, enum_con: Cow<'a, str>) -> Self {
        Self {
            tycon: Box::new(tycon),
            enum_con,
        }
    }

    pub fn tycon(&self) -> &DamlTyConName<'a> {
        &self.tycon
    }

    pub fn enum_con(&self) -> &str {
        &self.enum_con
    }
}

impl<'a> DamlVisitableElement<'a> for DamlEnumCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_enum_con(self);
        self.tycon.accept(visitor);
        visitor.post_visit_enum_con(self);
    }
}

impl ToStatic for DamlEnumCon<'_> {
    type Static = DamlEnumCon<'static>;

    fn to_static(&self) -> Self::Static {
        DamlEnumCon::new(self.tycon.to_static(), self.enum_con.to_static())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlStructCon<'a> {
    fields: Vec<DamlFieldWithExpr<'a>>,
}

impl<'a> DamlStructCon<'a> {
    pub fn new(fields: Vec<DamlFieldWithExpr<'a>>) -> Self {
        Self {
            fields,
        }
    }

    pub fn fields(&self) -> &[DamlFieldWithExpr<'a>] {
        &self.fields
    }
}

impl<'a> DamlVisitableElement<'a> for DamlStructCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_struct_con(self);
        self.fields.iter().for_each(|field| field.accept(visitor));
        visitor.post_visit_struct_con(self);
    }
}

impl ToStatic for DamlStructCon<'_> {
    type Static = DamlStructCon<'static>;

    fn to_static(&self) -> Self::Static {
        DamlStructCon::new(self.fields.iter().map(DamlFieldWithExpr::to_static).collect())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlStructProj<'a> {
    struct_expr: Box<DamlExpr<'a>>,
    field: Cow<'a, str>,
}

impl<'a> DamlStructProj<'a> {
    pub fn new(struct_expr: Box<DamlExpr<'a>>, field: Cow<'a, str>) -> Self {
        Self {
            struct_expr,
            field,
        }
    }

    pub fn struct_expr(&self) -> &DamlExpr<'a> {
        self.struct_expr.as_ref()
    }

    pub fn field(&self) -> &str {
        &self.field
    }
}

impl<'a> DamlVisitableElement<'a> for DamlStructProj<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_struct_proj(self);
        self.struct_expr.accept(visitor);
        visitor.post_visit_struct_proj(self);
    }
}

impl ToStatic for DamlStructProj<'_> {
    type Static = DamlStructProj<'static>;

    fn to_static(&self) -> Self::Static {
        DamlStructProj::new(Box::new(self.struct_expr.to_static()), self.field.to_static())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlStructUpd<'a> {
    struct_expr: Box<DamlExpr<'a>>,
    update: Box<DamlExpr<'a>>,
    field: Cow<'a, str>,
}

impl<'a> DamlStructUpd<'a> {
    pub fn new(struct_expr: Box<DamlExpr<'a>>, update: Box<DamlExpr<'a>>, field: Cow<'a, str>) -> Self {
        Self {
            struct_expr,
            update,
            field,
        }
    }

    pub fn struct_expr(&self) -> &DamlExpr<'a> {
        self.struct_expr.as_ref()
    }

    pub fn update(&self) -> &DamlExpr<'a> {
        self.update.as_ref()
    }

    pub fn field(&self) -> &str {
        &self.field
    }
}

impl<'a> DamlVisitableElement<'a> for DamlStructUpd<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_struct_upd(self);
        self.struct_expr.accept(visitor);
        self.update.accept(visitor);
        visitor.post_visit_struct_upd(self);
    }
}

impl ToStatic for DamlStructUpd<'_> {
    type Static = DamlStructUpd<'static>;

    fn to_static(&self) -> Self::Static {
        DamlStructUpd::new(
            Box::new(self.struct_expr.to_static()),
            Box::new(self.update.to_static()),
            self.field.to_static(),
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlApp<'a> {
    fun: Box<DamlExpr<'a>>,
    args: Vec<DamlExpr<'a>>,
}

impl<'a> DamlApp<'a> {
    pub fn new(fun: Box<DamlExpr<'a>>, args: Vec<DamlExpr<'a>>) -> Self {
        Self {
            fun,
            args,
        }
    }

    pub fn fun(&self) -> &DamlExpr<'a> {
        self.fun.as_ref()
    }

    pub fn args(&self) -> &[DamlExpr<'a>] {
        &self.args
    }
}

impl<'a> DamlVisitableElement<'a> for DamlApp<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_app(self);
        self.fun.accept(visitor);
        self.args.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_app(self);
    }
}

impl ToStatic for DamlApp<'_> {
    type Static = DamlApp<'static>;

    fn to_static(&self) -> Self::Static {
        DamlApp::new(Box::new(self.fun.to_static()), self.args.iter().map(DamlExpr::to_static).collect())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlTyApp<'a> {
    expr: Box<DamlExpr<'a>>,
    types: Vec<DamlType<'a>>,
}

impl<'a> DamlTyApp<'a> {
    pub fn new(expr: Box<DamlExpr<'a>>, types: Vec<DamlType<'a>>) -> Self {
        Self {
            expr,
            types,
        }
    }

    pub fn expr(&self) -> &DamlExpr<'a> {
        self.expr.as_ref()
    }

    pub fn types(&self) -> &[DamlType<'a>] {
        &self.types
    }
}

impl<'a> DamlVisitableElement<'a> for DamlTyApp<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_ty_app(self);
        self.expr.accept(visitor);
        self.types.iter().for_each(|ty| ty.accept(visitor));
        visitor.post_visit_ty_app(self);
    }
}

impl ToStatic for DamlTyApp<'_> {
    type Static = DamlTyApp<'static>;

    fn to_static(&self) -> Self::Static {
        DamlTyApp::new(Box::new(self.expr.to_static()), self.types.iter().map(DamlType::to_static).collect())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlAbs<'a> {
    params: Vec<DamlVarWithType<'a>>,
    body: Box<DamlExpr<'a>>,
}

impl<'a> DamlAbs<'a> {
    pub fn new(params: Vec<DamlVarWithType<'a>>, body: Box<DamlExpr<'a>>) -> Self {
        Self {
            params,
            body,
        }
    }

    pub fn params(&self) -> &[DamlVarWithType<'a>] {
        &self.params
    }

    pub fn body(&self) -> &DamlExpr<'a> {
        self.body.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlAbs<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_abs(self);
        self.params.iter().for_each(|param| param.accept(visitor));
        self.body.accept(visitor);
        visitor.post_visit_abs(self);
    }
}

impl ToStatic for DamlAbs<'_> {
    type Static = DamlAbs<'static>;

    fn to_static(&self) -> Self::Static {
        DamlAbs::new(self.params.iter().map(DamlVarWithType::to_static).collect(), Box::new(self.body.to_static()))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlVarWithType<'a> {
    ty: DamlType<'a>,
    var: Cow<'a, str>,
}

impl<'a> DamlVarWithType<'a> {
    pub fn new(ty: DamlType<'a>, var: Cow<'a, str>) -> Self {
        Self {
            ty,
            var,
        }
    }

    pub fn ty(&self) -> &DamlType<'a> {
        &self.ty
    }

    pub fn var(&self) -> &str {
        &self.var
    }
}

impl<'a> DamlVisitableElement<'a> for DamlVarWithType<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_var_with_type(self);
        self.ty.accept(visitor);
        visitor.post_visit_var_with_type(self);
    }
}

impl ToStatic for DamlVarWithType<'_> {
    type Static = DamlVarWithType<'static>;

    fn to_static(&self) -> Self::Static {
        DamlVarWithType::new(self.ty.to_static(), self.var.to_static())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlTyAbs<'a> {
    params: Vec<DamlTypeVarWithKind<'a>>,
    body: Box<DamlExpr<'a>>,
}

impl<'a> DamlTyAbs<'a> {
    pub fn new(params: Vec<DamlTypeVarWithKind<'a>>, body: Box<DamlExpr<'a>>) -> Self {
        Self {
            params,
            body,
        }
    }

    pub fn params(&self) -> &[DamlTypeVarWithKind<'a>] {
        &self.params
    }

    pub fn body(&self) -> &DamlExpr<'a> {
        self.body.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlTyAbs<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_ty_abs(self);
        self.params.iter().for_each(|type_var| type_var.accept(visitor));
        self.body.accept(visitor);
        visitor.post_visit_ty_abs(self);
    }
}

impl ToStatic for DamlTyAbs<'_> {
    type Static = DamlTyAbs<'static>;

    fn to_static(&self) -> Self::Static {
        DamlTyAbs::new(
            self.params.iter().map(DamlTypeVarWithKind::to_static).collect(),
            Box::new(self.body.to_static()),
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlBlock<'a> {
    bindings: Vec<DamlBinding<'a>>,
    body: Box<DamlExpr<'a>>,
}

impl<'a> DamlBlock<'a> {
    pub fn new(bindings: Vec<DamlBinding<'a>>, body: Box<DamlExpr<'a>>) -> Self {
        Self {
            bindings,
            body,
        }
    }

    pub fn bindings(&self) -> &[DamlBinding<'a>] {
        &self.bindings
    }

    pub fn body(&self) -> &DamlExpr<'a> {
        self.body.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlBlock<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_block(self);
        self.bindings.iter().for_each(|binding| binding.accept(visitor));
        self.body.accept(visitor);
        visitor.post_visit_block(self);
    }
}

impl ToStatic for DamlBlock<'_> {
    type Static = DamlBlock<'static>;

    fn to_static(&self) -> Self::Static {
        DamlBlock::new(self.bindings.iter().map(DamlBinding::to_static).collect(), Box::new(self.body.to_static()))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlBinding<'a> {
    binder: DamlVarWithType<'a>,
    bound: DamlExpr<'a>,
}

impl<'a> DamlBinding<'a> {
    pub fn new(binder: DamlVarWithType<'a>, bound: DamlExpr<'a>) -> Self {
        Self {
            binder,
            bound,
        }
    }

    pub fn binder(&self) -> &DamlVarWithType<'a> {
        &self.binder
    }

    pub fn bound(&self) -> &DamlExpr<'a> {
        &self.bound
    }
}

impl<'a> DamlVisitableElement<'a> for DamlBinding<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_binding(self);
        self.binder.accept(visitor);
        self.bound.accept(visitor);
        visitor.post_visit_binding(self);
    }
}

impl ToStatic for DamlBinding<'_> {
    type Static = DamlBinding<'static>;

    fn to_static(&self) -> Self::Static {
        DamlBinding::new(self.binder.to_static(), self.bound.to_static())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlCons<'a> {
    ty: DamlType<'a>,
    front: Vec<DamlExpr<'a>>,
    tail: Box<DamlExpr<'a>>,
}

impl<'a> DamlCons<'a> {
    pub fn new(ty: DamlType<'a>, front: Vec<DamlExpr<'a>>, tail: Box<DamlExpr<'a>>) -> Self {
        Self {
            ty,
            front,
            tail,
        }
    }

    pub fn ty(&self) -> &DamlType<'a> {
        &self.ty
    }

    pub fn front(&self) -> &[DamlExpr<'a>] {
        &self.front
    }

    pub fn tail(&self) -> &DamlExpr<'a> {
        self.tail.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCons<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_cons(self);
        self.ty.accept(visitor);
        self.front.iter().for_each(|expr| expr.accept(visitor));
        self.tail.accept(visitor);
        visitor.post_visit_cons(self);
    }
}

impl ToStatic for DamlCons<'_> {
    type Static = DamlCons<'static>;

    fn to_static(&self) -> Self::Static {
        DamlCons::new(
            self.ty.to_static(),
            self.front.iter().map(DamlExpr::to_static).collect(),
            Box::new(self.tail.to_static()),
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlCase<'a> {
    scrut: Box<DamlExpr<'a>>,
    alts: Vec<DamlCaseAlt<'a>>,
}

impl<'a> DamlCase<'a> {
    pub fn new(scrut: Box<DamlExpr<'a>>, alts: Vec<DamlCaseAlt<'a>>) -> Self {
        Self {
            scrut,
            alts,
        }
    }

    pub fn scrut(&self) -> &DamlExpr<'a> {
        self.scrut.as_ref()
    }

    pub fn alts(&self) -> &[DamlCaseAlt<'a>] {
        &self.alts
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCase<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_case(self);
        self.scrut.accept(visitor);
        self.alts.iter().for_each(|alt| alt.accept(visitor));
        visitor.post_visit_case(self);
    }
}

impl ToStatic for DamlCase<'_> {
    type Static = DamlCase<'static>;

    fn to_static(&self) -> Self::Static {
        DamlCase::new(Box::new(self.scrut.to_static()), self.alts.iter().map(DamlCaseAlt::to_static).collect())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlCaseAlt<'a> {
    body: DamlExpr<'a>,
    sum: DamlCaseAltSum<'a>,
}

impl<'a> DamlCaseAlt<'a> {
    pub fn new(body: DamlExpr<'a>, sum: DamlCaseAltSum<'a>) -> Self {
        Self {
            body,
            sum,
        }
    }

    pub fn body(&self) -> &DamlExpr<'a> {
        &self.body
    }

    pub fn sum(&self) -> &DamlCaseAltSum<'a> {
        &self.sum
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCaseAlt<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_case_alt(self);
        self.body.accept(visitor);
        self.sum.accept(visitor);
        visitor.post_visit_case_alt(self);
    }
}

impl ToStatic for DamlCaseAlt<'_> {
    type Static = DamlCaseAlt<'static>;

    fn to_static(&self) -> Self::Static {
        DamlCaseAlt::new(self.body.to_static(), self.sum.to_static())
    }
}

#[derive(Debug, Serialize, Clone)]
#[allow(clippy::large_enum_variant)] // TODO look at why DamlCaseAltVariant is so large (280 bytes!)
pub enum DamlCaseAltSum<'a> {
    Default,
    Variant(DamlCaseAltVariant<'a>),
    PrimCon(DamlPrimCon),
    Nil,
    Cons(DamlCaseAltCons<'a>),
    OptionalNone,
    OptionalSome(DamlCaseAltOptionalSome<'a>),
    Enum(DamlCaseAltEnum<'a>),
}

impl<'a> DamlVisitableElement<'a> for DamlCaseAltSum<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_case_alt_sum(self);
        match self {
            DamlCaseAltSum::Variant(variant) => variant.accept(visitor),
            DamlCaseAltSum::PrimCon(prim_con) => prim_con.accept(visitor),
            DamlCaseAltSum::Cons(cons) => cons.accept(visitor),
            DamlCaseAltSum::OptionalSome(opt_some) => opt_some.accept(visitor),
            DamlCaseAltSum::Enum(enum_alt) => enum_alt.accept(visitor),
            DamlCaseAltSum::Default | DamlCaseAltSum::Nil | DamlCaseAltSum::OptionalNone => {},
        }
        visitor.post_visit_case_alt_sum(self);
    }
}

impl ToStatic for DamlCaseAltSum<'_> {
    type Static = DamlCaseAltSum<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            DamlCaseAltSum::Default => DamlCaseAltSum::Default,
            DamlCaseAltSum::Variant(variant) => DamlCaseAltSum::Variant(variant.to_static()),
            DamlCaseAltSum::PrimCon(prim_con) => DamlCaseAltSum::PrimCon(*prim_con),
            DamlCaseAltSum::Nil => DamlCaseAltSum::Nil,
            DamlCaseAltSum::Cons(cons) => DamlCaseAltSum::Cons(cons.to_static()),
            DamlCaseAltSum::OptionalNone => DamlCaseAltSum::OptionalNone,
            DamlCaseAltSum::OptionalSome(opt_some) => DamlCaseAltSum::OptionalSome(opt_some.to_static()),
            DamlCaseAltSum::Enum(enum_alt) => DamlCaseAltSum::Enum(enum_alt.to_static()),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlCaseAltVariant<'a> {
    con: DamlTyConName<'a>,
    variant: Cow<'a, str>,
    binder: Cow<'a, str>,
}

impl<'a> DamlCaseAltVariant<'a> {
    pub fn new(con: DamlTyConName<'a>, variant: Cow<'a, str>, binder: Cow<'a, str>) -> Self {
        Self {
            con,
            variant,
            binder,
        }
    }

    pub fn con(&self) -> &DamlTyConName<'a> {
        &self.con
    }

    pub fn variant(&self) -> &str {
        &self.variant
    }

    pub fn binder(&self) -> &str {
        &self.binder
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCaseAltVariant<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_case_alt_variant(self);
        self.con.accept(visitor);
        visitor.post_visit_case_alt_variant(self);
    }
}

impl ToStatic for DamlCaseAltVariant<'_> {
    type Static = DamlCaseAltVariant<'static>;

    fn to_static(&self) -> Self::Static {
        DamlCaseAltVariant::new(self.con.to_static(), self.variant.to_static(), self.binder.to_static())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlCaseAltCons<'a> {
    var_head: Cow<'a, str>,
    var_tail: Cow<'a, str>,
}

impl<'a> DamlCaseAltCons<'a> {
    pub fn new(var_head: Cow<'a, str>, var_tail: Cow<'a, str>) -> Self {
        Self {
            var_head,
            var_tail,
        }
    }

    pub fn var_head(&self) -> &str {
        &self.var_head
    }

    pub fn var_tail(&self) -> &str {
        &self.var_tail
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCaseAltCons<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_case_alt_cons(self);
        visitor.post_visit_case_alt_cons(self);
    }
}

impl ToStatic for DamlCaseAltCons<'_> {
    type Static = DamlCaseAltCons<'static>;

    fn to_static(&self) -> Self::Static {
        DamlCaseAltCons::new(self.var_head.to_static(), self.var_tail.to_static())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlCaseAltOptionalSome<'a> {
    var_body: Cow<'a, str>,
}

impl<'a> DamlCaseAltOptionalSome<'a> {
    pub fn new(var_body: Cow<'a, str>) -> Self {
        Self {
            var_body,
        }
    }

    pub fn var_body(&self) -> &str {
        &self.var_body
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCaseAltOptionalSome<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_case_alt_opt_some(self);
        visitor.post_visit_case_alt_opt_some(self);
    }
}

impl ToStatic for DamlCaseAltOptionalSome<'_> {
    type Static = DamlCaseAltOptionalSome<'static>;

    fn to_static(&self) -> Self::Static {
        DamlCaseAltOptionalSome::new(self.var_body.to_static())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlCaseAltEnum<'a> {
    con: DamlTyConName<'a>,
    constructor: Cow<'a, str>,
}

impl<'a> DamlCaseAltEnum<'a> {
    pub fn new(con: DamlTyConName<'a>, constructor: Cow<'a, str>) -> Self {
        Self {
            con,
            constructor,
        }
    }

    pub fn con(&self) -> &DamlTyConName<'a> {
        &self.con
    }

    pub fn constructor(&self) -> &str {
        &self.constructor
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCaseAltEnum<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_case_alt_enum(self);
        self.con.accept(visitor);
        visitor.post_visit_case_alt_enum(self);
    }
}

impl ToStatic for DamlCaseAltEnum<'_> {
    type Static = DamlCaseAltEnum<'static>;

    fn to_static(&self) -> Self::Static {
        DamlCaseAltEnum::new(self.con.to_static(), self.constructor.to_static())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlOptionalSome<'a> {
    ty: DamlType<'a>,
    body: Box<DamlExpr<'a>>,
}

impl<'a> DamlOptionalSome<'a> {
    pub fn new(ty: DamlType<'a>, body: Box<DamlExpr<'a>>) -> Self {
        Self {
            ty,
            body,
        }
    }

    pub fn ty(&self) -> &DamlType<'a> {
        &self.ty
    }

    pub fn body(&self) -> &DamlExpr<'a> {
        self.body.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlOptionalSome<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_optional_some(self);
        self.ty.accept(visitor);
        self.body.accept(visitor);
        visitor.post_visit_optional_some(self);
    }
}

impl ToStatic for DamlOptionalSome<'_> {
    type Static = DamlOptionalSome<'static>;

    fn to_static(&self) -> Self::Static {
        DamlOptionalSome::new(self.ty.to_static(), Box::new(self.body.to_static()))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlToAny<'a> {
    ty: DamlType<'a>,
    expr: Box<DamlExpr<'a>>,
}

impl<'a> DamlToAny<'a> {
    pub fn new(ty: DamlType<'a>, expr: Box<DamlExpr<'a>>) -> Self {
        Self {
            ty,
            expr,
        }
    }

    pub fn ty(&self) -> &DamlType<'a> {
        &self.ty
    }

    pub fn expr(&self) -> &DamlExpr<'a> {
        self.expr.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlToAny<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_to_any(self);
        self.ty.accept(visitor);
        self.expr.accept(visitor);
        visitor.post_visit_to_any(self);
    }
}

impl ToStatic for DamlToAny<'_> {
    type Static = DamlToAny<'static>;

    fn to_static(&self) -> Self::Static {
        DamlToAny::new(self.ty.to_static(), Box::new(self.expr.to_static()))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlFromAny<'a> {
    ty: DamlType<'a>,
    expr: Box<DamlExpr<'a>>,
}

impl<'a> DamlFromAny<'a> {
    pub fn new(ty: DamlType<'a>, expr: Box<DamlExpr<'a>>) -> Self {
        Self {
            ty,
            expr,
        }
    }

    pub fn ty(&self) -> &DamlType<'a> {
        &self.ty
    }

    pub fn expr(&self) -> &DamlExpr<'a> {
        self.expr.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlFromAny<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_from_any(self);
        self.ty.accept(visitor);
        self.expr.accept(visitor);
        visitor.post_visit_from_any(self);
    }
}

impl ToStatic for DamlFromAny<'_> {
    type Static = DamlFromAny<'static>;

    fn to_static(&self) -> Self::Static {
        DamlFromAny::new(self.ty.to_static(), Box::new(self.expr.to_static()))
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum DamlUpdate<'a> {
    Pure(DamlPure<'a>),
    Block(DamlBlock<'a>),
    Create(DamlCreate<'a>),
    Exercise(DamlExercise<'a>),
    ExerciseByKey(DamlExerciseByKey<'a>),
    Fetch(DamlFetch<'a>),
    GetTime,
    LookupByKey(DamlRetrieveByKey<'a>),
    FetchByKey(DamlRetrieveByKey<'a>),
    EmbedExpr(DamlUpdateEmbedExpr<'a>),
}

impl<'a> DamlVisitableElement<'a> for DamlUpdate<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_update(self);
        match self {
            DamlUpdate::Pure(pure) => pure.accept(visitor),
            DamlUpdate::Block(block) => block.accept(visitor),
            DamlUpdate::Create(create) => create.accept(visitor),
            DamlUpdate::Exercise(exercise) => exercise.accept(visitor),
            DamlUpdate::ExerciseByKey(exercise_by_key) => exercise_by_key.accept(visitor),
            DamlUpdate::Fetch(fetch) => fetch.accept(visitor),
            DamlUpdate::LookupByKey(retrieve_by_key) | DamlUpdate::FetchByKey(retrieve_by_key) =>
                retrieve_by_key.accept(visitor),
            DamlUpdate::EmbedExpr(embed_expr) => embed_expr.accept(visitor),
            DamlUpdate::GetTime => {},
        }
        visitor.post_visit_update(self);
    }
}

impl ToStatic for DamlUpdate<'_> {
    type Static = DamlUpdate<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            DamlUpdate::Pure(pure) => DamlUpdate::Pure(pure.to_static()),
            DamlUpdate::Block(block) => DamlUpdate::Block(block.to_static()),
            DamlUpdate::Create(create) => DamlUpdate::Create(create.to_static()),
            DamlUpdate::Exercise(exercise) => DamlUpdate::Exercise(exercise.to_static()),
            DamlUpdate::ExerciseByKey(exercise_by_key) => DamlUpdate::ExerciseByKey(exercise_by_key.to_static()),
            DamlUpdate::Fetch(fetch) => DamlUpdate::Fetch(fetch.to_static()),
            DamlUpdate::GetTime => DamlUpdate::GetTime,
            DamlUpdate::LookupByKey(retrieve_by_key) => DamlUpdate::LookupByKey(retrieve_by_key.to_static()),
            DamlUpdate::FetchByKey(retrieve_by_key) => DamlUpdate::FetchByKey(retrieve_by_key.to_static()),
            DamlUpdate::EmbedExpr(embed_expr) => DamlUpdate::EmbedExpr(embed_expr.to_static()),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlPure<'a> {
    ty: DamlType<'a>,
    expr: Box<DamlExpr<'a>>,
}

impl<'a> DamlPure<'a> {
    pub fn new(ty: DamlType<'a>, expr: Box<DamlExpr<'a>>) -> Self {
        Self {
            ty,
            expr,
        }
    }

    pub fn ty(&self) -> &DamlType<'a> {
        &self.ty
    }

    pub fn expr(&self) -> &DamlExpr<'a> {
        self.expr.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlPure<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_pure(self);
        self.ty.accept(visitor);
        self.expr.accept(visitor);
        visitor.post_visit_pure(self);
    }
}

impl ToStatic for DamlPure<'_> {
    type Static = DamlPure<'static>;

    fn to_static(&self) -> Self::Static {
        DamlPure::new(self.ty.to_static(), Box::new(self.expr.to_static()))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlCreate<'a> {
    template: Box<DamlTyConName<'a>>,
    expr: Box<DamlExpr<'a>>,
}

impl<'a> DamlCreate<'a> {
    pub fn new(template: DamlTyConName<'a>, expr: Box<DamlExpr<'a>>) -> Self {
        Self {
            template: Box::new(template),
            expr,
        }
    }

    pub fn template(&self) -> &DamlTyConName<'a> {
        &self.template
    }

    pub fn expr(&self) -> &DamlExpr<'a> {
        self.expr.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCreate<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_create(self);
        self.template.accept(visitor);
        self.expr.accept(visitor);
        visitor.post_visit_create(self);
    }
}

impl ToStatic for DamlCreate<'_> {
    type Static = DamlCreate<'static>;

    fn to_static(&self) -> Self::Static {
        DamlCreate::new(self.template.to_static(), Box::new(self.expr.to_static()))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlExercise<'a> {
    template: Box<DamlTyConName<'a>>,
    cid: Box<DamlExpr<'a>>,
    arg: Box<DamlExpr<'a>>,
    choice: Cow<'a, str>,
}

impl<'a> DamlExercise<'a> {
    pub fn new(
        template: DamlTyConName<'a>,
        cid: Box<DamlExpr<'a>>,
        arg: Box<DamlExpr<'a>>,
        choice: Cow<'a, str>,
    ) -> Self {
        Self {
            template: Box::new(template),
            cid,
            arg,
            choice,
        }
    }

    pub fn template(&self) -> &DamlTyConName<'a> {
        &self.template
    }

    pub fn cid(&self) -> &DamlExpr<'a> {
        self.cid.as_ref()
    }

    pub fn arg(&self) -> &DamlExpr<'a> {
        self.arg.as_ref()
    }

    pub fn choice(&self) -> &str {
        &self.choice
    }
}

impl<'a> DamlVisitableElement<'a> for DamlExercise<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_exercise(self);
        self.template.accept(visitor);
        self.cid.accept(visitor);
        self.arg.accept(visitor);
        visitor.post_visit_exercise(self);
    }
}

impl ToStatic for DamlExercise<'_> {
    type Static = DamlExercise<'static>;

    fn to_static(&self) -> Self::Static {
        DamlExercise::new(
            self.template.to_static(),
            Box::new(self.cid.to_static()),
            Box::new(self.arg.to_static()),
            self.choice.to_static(),
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlExerciseByKey<'a> {
    template: Box<DamlTyConName<'a>>,
    choice: Cow<'a, str>,
    key: Box<DamlExpr<'a>>,
    arg: Box<DamlExpr<'a>>,
}

impl<'a> DamlExerciseByKey<'a> {
    pub fn new(
        template: DamlTyConName<'a>,
        choice: Cow<'a, str>,
        key: Box<DamlExpr<'a>>,
        arg: Box<DamlExpr<'a>>,
    ) -> Self {
        Self {
            template: Box::new(template),
            choice,
            key,
            arg,
        }
    }

    pub fn template(&self) -> &DamlTyConName<'a> {
        &self.template
    }

    pub fn choice(&self) -> &str {
        &self.choice
    }

    pub fn key(&self) -> &DamlExpr<'a> {
        self.key.as_ref()
    }

    pub fn arg(&self) -> &DamlExpr<'a> {
        self.arg.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlExerciseByKey<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_exercise_by_key(self);
        self.template.accept(visitor);
        self.key.accept(visitor);
        self.arg.accept(visitor);
        visitor.post_visit_exercise_by_key(self);
    }
}

impl ToStatic for DamlExerciseByKey<'_> {
    type Static = DamlExerciseByKey<'static>;

    fn to_static(&self) -> Self::Static {
        DamlExerciseByKey::new(
            self.template.to_static(),
            self.choice.to_static(),
            Box::new(self.key.to_static()),
            Box::new(self.arg.to_static()),
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlFetch<'a> {
    template: Box<DamlTyConName<'a>>,
    cid: Box<DamlExpr<'a>>,
}

impl<'a> DamlFetch<'a> {
    pub fn new(template: DamlTyConName<'a>, cid: Box<DamlExpr<'a>>) -> Self {
        Self {
            template: Box::new(template),
            cid,
        }
    }

    pub fn template(&self) -> &DamlTyConName<'a> {
        &self.template
    }

    pub fn cid(&self) -> &DamlExpr<'a> {
        self.cid.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlFetch<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_fetch(self);
        self.template.accept(visitor);
        self.cid.accept(visitor);
        visitor.post_visit_fetch(self);
    }
}

impl ToStatic for DamlFetch<'_> {
    type Static = DamlFetch<'static>;

    fn to_static(&self) -> Self::Static {
        DamlFetch::new(self.template.to_static(), Box::new(self.cid.to_static()))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlRetrieveByKey<'a> {
    template: Box<DamlTyConName<'a>>,
    key: Box<DamlExpr<'a>>,
}

impl<'a> DamlRetrieveByKey<'a> {
    pub fn new(template: DamlTyConName<'a>, key: Box<DamlExpr<'a>>) -> Self {
        Self {
            template: Box::new(template),
            key,
        }
    }

    pub fn template(&self) -> &DamlTyConName<'a> {
        &self.template
    }

    pub fn key(&self) -> &DamlExpr<'a> {
        self.key.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlRetrieveByKey<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_retrieve_by_key(self);
        self.template.accept(visitor);
        self.key.accept(visitor);
        visitor.post_visit_retrieve_by_key(self);
    }
}

impl ToStatic for DamlRetrieveByKey<'_> {
    type Static = DamlRetrieveByKey<'static>;

    fn to_static(&self) -> Self::Static {
        DamlRetrieveByKey::new(self.template.to_static(), Box::new(self.key.to_static()))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlUpdateEmbedExpr<'a> {
    ty: DamlType<'a>,
    body: Box<DamlExpr<'a>>,
}

impl<'a> DamlUpdateEmbedExpr<'a> {
    pub fn new(ty: DamlType<'a>, body: Box<DamlExpr<'a>>) -> Self {
        Self {
            ty,
            body,
        }
    }

    pub fn ty(&self) -> &DamlType<'a> {
        &self.ty
    }

    pub fn body(&self) -> &DamlExpr<'a> {
        self.body.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlUpdateEmbedExpr<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_update_embed_expr(self);
        self.ty.accept(visitor);
        self.body.accept(visitor);
        visitor.post_visit_update_embed_expr(self);
    }
}

impl ToStatic for DamlUpdateEmbedExpr<'_> {
    type Static = DamlUpdateEmbedExpr<'static>;

    fn to_static(&self) -> Self::Static {
        DamlUpdateEmbedExpr::new(self.ty.to_static(), Box::new(self.body.to_static()))
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum DamlScenario<'a> {
    Pure(DamlPure<'a>),
    Block(DamlBlock<'a>),
    Commit(DamlCommit<'a>),
    MustFailAt(DamlCommit<'a>),
    Pass(Box<DamlExpr<'a>>),
    GetTime,
    GetParty(Box<DamlExpr<'a>>),
    EmbedExpr(DamlScenarioEmbedExpr<'a>),
}

impl<'a> DamlVisitableElement<'a> for DamlScenario<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_scenario(self);
        match self {
            DamlScenario::Pure(pure) => pure.accept(visitor),
            DamlScenario::Block(block) => block.accept(visitor),
            DamlScenario::Commit(commit) | DamlScenario::MustFailAt(commit) => commit.accept(visitor),
            DamlScenario::Pass(expr) | DamlScenario::GetParty(expr) => expr.accept(visitor),
            DamlScenario::GetTime => {},
            DamlScenario::EmbedExpr(embed_expr) => embed_expr.accept(visitor),
        }
        visitor.post_visit_scenario(self);
    }
}

impl ToStatic for DamlScenario<'_> {
    type Static = DamlScenario<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            DamlScenario::Pure(pure) => DamlScenario::Pure(pure.to_static()),
            DamlScenario::Block(block) => DamlScenario::Block(block.to_static()),
            DamlScenario::Commit(commit) => DamlScenario::Commit(commit.to_static()),
            DamlScenario::MustFailAt(commit) => DamlScenario::MustFailAt(commit.to_static()),
            DamlScenario::Pass(expr) => DamlScenario::Pass(Box::new(expr.to_static())),
            DamlScenario::GetTime => DamlScenario::GetTime,
            DamlScenario::GetParty(expr) => DamlScenario::GetParty(Box::new(expr.to_static())),
            DamlScenario::EmbedExpr(embed_expr) => DamlScenario::EmbedExpr(embed_expr.to_static()),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlCommit<'a> {
    party: Box<DamlExpr<'a>>,
    expr: Box<DamlExpr<'a>>,
    ret_type: DamlType<'a>,
}

impl<'a> DamlCommit<'a> {
    pub fn new(party: Box<DamlExpr<'a>>, expr: Box<DamlExpr<'a>>, ret_type: DamlType<'a>) -> Self {
        Self {
            party,
            expr,
            ret_type,
        }
    }

    pub fn party(&self) -> &DamlExpr<'a> {
        self.party.as_ref()
    }

    pub fn expr(&self) -> &DamlExpr<'a> {
        self.expr.as_ref()
    }

    pub fn ret_type(&self) -> &DamlType<'a> {
        &self.ret_type
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCommit<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_commit(self);
        self.party.accept(visitor);
        self.expr.accept(visitor);
        self.ret_type.accept(visitor);
        visitor.post_visit_commit(self);
    }
}

impl ToStatic for DamlCommit<'_> {
    type Static = DamlCommit<'static>;

    fn to_static(&self) -> Self::Static {
        DamlCommit::new(Box::new(self.party.to_static()), Box::new(self.expr.to_static()), self.ret_type.to_static())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlScenarioEmbedExpr<'a> {
    ty: DamlType<'a>,
    body: Box<DamlExpr<'a>>,
}

impl<'a> DamlScenarioEmbedExpr<'a> {
    pub fn new(ty: DamlType<'a>, body: Box<DamlExpr<'a>>) -> Self {
        Self {
            ty,
            body,
        }
    }

    pub fn ty(&self) -> &DamlType<'a> {
        &self.ty
    }

    pub fn body(&self) -> &DamlExpr<'a> {
        self.body.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlScenarioEmbedExpr<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_scenario_embed_expr(self);
        self.ty.accept(visitor);
        self.body.accept(visitor);
        visitor.post_visit_scenario_embed_expr(self);
    }
}

impl ToStatic for DamlScenarioEmbedExpr<'_> {
    type Static = DamlScenarioEmbedExpr<'static>;

    fn to_static(&self) -> Self::Static {
        DamlScenarioEmbedExpr::new(self.ty.to_static(), Box::new(self.body.to_static()))
    }
}
