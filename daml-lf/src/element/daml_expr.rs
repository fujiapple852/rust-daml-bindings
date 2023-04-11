use crate::element::{
    DamlElementVisitor, DamlTyCon, DamlTyConName, DamlType, DamlTypeVarWithKind, DamlVisitableElement,
};
use bounded_static::ToStatic;
use serde::Serialize;
use std::borrow::Cow;
use std::fmt;
use std::fmt::{Display, Formatter};

/// A Daml expression.
#[derive(Debug, Serialize, Clone, ToStatic)]
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
    ToAnyException(DamlToAnyException<'a>),
    FromAnyException(DamlFromAnyException<'a>),
    Throw(DamlThrow<'a>),
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
            DamlExpr::ToAnyException(to_any_exception) => to_any_exception.accept(visitor),
            DamlExpr::FromAnyException(from_any_exception) => from_any_exception.accept(visitor),
            DamlExpr::Throw(throw) => throw.accept(visitor),
            DamlExpr::Var(_) => {},
        }
        visitor.post_visit_expr(self);
    }
}

/// A Daml value name.
#[derive(Debug, Serialize, Clone, ToStatic)]
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
            DamlValueName::Local(local) => {
                write!(f, "{}:{}:{}", local.package_name, &local.module_path.join("."), local.name)
            },
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

/// A Daml local value name.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml non-local value name.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression builtin function.
#[derive(Debug, Serialize, Clone, ToStatic)]
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
    AnyExceptionMessage,
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
    Int64ToText,
    DecimalToText,
    NumericToText,
    TextToText,
    TimestampToText,
    DateToText,
    PartyToQuotedText,
    PartyToText,
    TextToParty,
    TextToInt64,
    TextToDecimal,
    TextToNumeric,
    ContractIdToText,
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
    CodePointsToText,
    TextPointsToCode,
    ScaleBignumeric,
    PrecisionBignumeric,
    AddBignumeric,
    SubBignumeric,
    MulBignumeric,
    DivBignumeric,
    ShiftBignumeric,
    ShiftRightBignumeric,
    BigNumericToNumeric,
    NumericToBigNumeric,
    BigNumericToText,
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

/// A Daml expression primitive constructor.
#[derive(Debug, Serialize, Copy, Clone, ToStatic)]
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

/// A Daml expression primitive literal.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression round mode.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression record constructor.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression field with expression.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression record projection.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression record update.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression variant constructor.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression enum constructor.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlEnumCon<'a> {
    tycon: Box<DamlTyConName<'a>>,
    enum_con: Cow<'a, str>,
}

impl<'a> DamlEnumCon<'a> {
    pub fn new(tycon: Box<DamlTyConName<'a>>, enum_con: Cow<'a, str>) -> Self {
        Self {
            tycon,
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

/// A Daml expression struct constructor.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression struct projection.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression struct update.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression function application.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression type application.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression variable abstraction.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression variable with type.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression type abstraction.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression block.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression binding.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression list constructor.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression case.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression case alternative.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression case sum type.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression case variant.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression case list constructor.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression case optional which is present.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression case enum.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression optional which is present.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression to any conversion.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression from any conversion.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression update effect.
#[derive(Debug, Serialize, Clone, ToStatic)]
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
    TryCatch(DamlTryCatch<'a>),
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
            DamlUpdate::TryCatch(try_catch) => try_catch.accept(visitor),
            DamlUpdate::GetTime => {},
        }
        visitor.post_visit_update(self);
    }
}

/// A Daml expression update pure.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression update effect create action.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlCreate<'a> {
    template: Box<DamlTyConName<'a>>,
    expr: Box<DamlExpr<'a>>,
}

impl<'a> DamlCreate<'a> {
    pub fn new(template: Box<DamlTyConName<'a>>, expr: Box<DamlExpr<'a>>) -> Self {
        Self {
            template,
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

/// A Daml expression update effect exercise action.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlExercise<'a> {
    template: Box<DamlTyConName<'a>>,
    cid: Box<DamlExpr<'a>>,
    arg: Box<DamlExpr<'a>>,
    choice: Cow<'a, str>,
}

impl<'a> DamlExercise<'a> {
    pub fn new(
        template: Box<DamlTyConName<'a>>,
        cid: Box<DamlExpr<'a>>,
        arg: Box<DamlExpr<'a>>,
        choice: Cow<'a, str>,
    ) -> Self {
        Self {
            template,
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

/// A Daml expression update effect exercise-by-key action.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlExerciseByKey<'a> {
    template: Box<DamlTyConName<'a>>,
    choice: Cow<'a, str>,
    key: Box<DamlExpr<'a>>,
    arg: Box<DamlExpr<'a>>,
}

impl<'a> DamlExerciseByKey<'a> {
    pub fn new(
        template: Box<DamlTyConName<'a>>,
        choice: Cow<'a, str>,
        key: Box<DamlExpr<'a>>,
        arg: Box<DamlExpr<'a>>,
    ) -> Self {
        Self {
            template,
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

/// A Daml expression update effect fetch action.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlFetch<'a> {
    template: Box<DamlTyConName<'a>>,
    cid: Box<DamlExpr<'a>>,
}

impl<'a> DamlFetch<'a> {
    pub fn new(template: Box<DamlTyConName<'a>>, cid: Box<DamlExpr<'a>>) -> Self {
        Self {
            template,
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

/// A Daml expression update effect retrieve-by-key action.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlRetrieveByKey<'a> {
    template: Box<DamlTyConName<'a>>,
    key: Box<DamlExpr<'a>>,
}

impl<'a> DamlRetrieveByKey<'a> {
    pub fn new(template: Box<DamlTyConName<'a>>, key: Box<DamlExpr<'a>>) -> Self {
        Self {
            template,
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

/// A Daml expression embedded update effect action.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression scenario effect.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression scenario commit action.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression embedded scenario expression.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml expression to any exception conversion.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlToAnyException<'a> {
    ty: DamlType<'a>,
    expr: Box<DamlExpr<'a>>,
}

impl<'a> DamlToAnyException<'a> {
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

impl<'a> DamlVisitableElement<'a> for DamlToAnyException<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_to_any_exception(self);
        self.ty.accept(visitor);
        self.expr.accept(visitor);
        visitor.post_visit_to_any_exception(self);
    }
}

/// A Daml expression from any exception conversion.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlFromAnyException<'a> {
    ty: DamlType<'a>,
    expr: Box<DamlExpr<'a>>,
}

impl<'a> DamlFromAnyException<'a> {
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

impl<'a> DamlVisitableElement<'a> for DamlFromAnyException<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_from_any_exception(self);
        self.ty.accept(visitor);
        self.expr.accept(visitor);
        visitor.post_visit_from_any_exception(self);
    }
}

/// A Daml expression throw exception.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlThrow<'a> {
    return_type: DamlType<'a>,
    exception_type: DamlType<'a>,
    exception_expr: Box<DamlExpr<'a>>,
}

impl<'a> DamlThrow<'a> {
    pub fn new(return_type: DamlType<'a>, exception_type: DamlType<'a>, exception_expr: Box<DamlExpr<'a>>) -> Self {
        Self {
            return_type,
            exception_type,
            exception_expr,
        }
    }

    pub fn return_type(&self) -> &DamlType<'a> {
        &self.return_type
    }

    pub fn exception_type(&self) -> &DamlType<'a> {
        &self.exception_type
    }

    pub fn exception_expr(&self) -> &DamlExpr<'a> {
        self.exception_expr.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlThrow<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_throw(self);
        self.return_type.accept(visitor);
        self.exception_type.accept(visitor);
        self.exception_expr.accept(visitor);
        visitor.post_visit_throw(self);
    }
}

/// A Daml expression update effect try/catch action.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlTryCatch<'a> {
    return_type: DamlType<'a>,
    try_expr: Box<DamlExpr<'a>>,
    var: Cow<'a, str>,
    catch_expr: Box<DamlExpr<'a>>,
}

impl<'a> DamlTryCatch<'a> {
    pub fn new(
        return_type: DamlType<'a>,
        try_expr: Box<DamlExpr<'a>>,
        var: Cow<'a, str>,
        catch_expr: Box<DamlExpr<'a>>,
    ) -> Self {
        Self {
            return_type,
            try_expr,
            var,
            catch_expr,
        }
    }

    pub fn return_type(&self) -> &DamlType<'a> {
        &self.return_type
    }

    pub fn try_expr(&self) -> &DamlExpr<'a> {
        self.try_expr.as_ref()
    }

    pub fn var(&self) -> &str {
        &self.var
    }

    pub fn catch_expr(&self) -> &DamlExpr<'a> {
        self.catch_expr.as_ref()
    }
}

impl<'a> DamlVisitableElement<'a> for DamlTryCatch<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_try_catch(self);
        self.return_type.accept(visitor);
        self.try_expr.accept(visitor);
        self.catch_expr.accept(visitor);
        visitor.post_visit_try_catch(self);
    }
}
