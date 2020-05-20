use crate::element::{
    DamlAbsoluteTyCon, DamlElementVisitor, DamlLocalTyCon, DamlNonLocalTyCon, DamlTyCon, DamlTyConName, DamlType,
    DamlTypeVarWithKind, DamlVisitableElement,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub enum DamlExpr<'a> {
    Var(&'a str),
    Val(DamlValueName<'a>),
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

#[derive(Debug, Serialize, Clone)]
pub enum DamlValueName<'a> {
    Local(DamlLocalTyCon<'a>),
    NonLocal(DamlNonLocalTyCon<'a>),
    Absolute(DamlAbsoluteTyCon<'a>),
}

impl<'a> DamlVisitableElement<'a> for DamlValueName<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_value_name(self);
        match self {
            DamlValueName::Local(local) => local.accept(visitor),
            DamlValueName::NonLocal(non_local) => non_local.accept(visitor),
            DamlValueName::Absolute(absolute) => absolute.accept(visitor),
        }
        visitor.post_visit_value_name(self);
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
}

impl<'a> DamlVisitableElement<'a> for DamlBuiltinFunction {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_builtin_function(self);
        visitor.post_visit_builtin_function(self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum DamlPrimCon {
    Unit,
    False,
    True,
}

impl<'a> DamlVisitableElement<'a> for DamlPrimCon {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_prim_con(self);
        visitor.post_visit_prim_con(self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum DamlPrimLit<'a> {
    /// Represents a standard signed 64-bit integer (integer between −2⁶³ to 2⁶³−1).
    Int64(i64),
    /// Represents a UTF8 string.
    Text(&'a str),
    /// A LitParty represents a party.
    Party(&'a str),
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
    Numeric(&'a str),
}

impl<'a> DamlVisitableElement<'a> for DamlPrimLit<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_prim_lit(self);
        visitor.post_visit_prim_lit(self);
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

#[derive(Debug, Serialize, Clone)]
pub struct DamlFieldWithExpr<'a> {
    field: &'a str,
    expr: DamlExpr<'a>,
}

impl<'a> DamlFieldWithExpr<'a> {
    pub fn new(field: &'a str, expr: DamlExpr<'a>) -> Self {
        Self {
            field,
            expr,
        }
    }

    pub fn field(&self) -> &str {
        self.field
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

#[derive(Debug, Serialize, Clone)]
pub struct DamlRecProj<'a> {
    tycon: DamlTyCon<'a>,
    record: Box<DamlExpr<'a>>,
    field: &'a str,
}

impl<'a> DamlRecProj<'a> {
    pub fn new(tycon: DamlTyCon<'a>, record: Box<DamlExpr<'a>>, field: &'a str) -> Self {
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
        self.field
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

#[derive(Debug, Serialize, Clone)]
pub struct DamlRecUpd<'a> {
    tycon: DamlTyCon<'a>,
    record: Box<DamlExpr<'a>>,
    update: Box<DamlExpr<'a>>,
    field: &'a str,
}

impl<'a> DamlRecUpd<'a> {
    pub fn new(tycon: DamlTyCon<'a>, record: Box<DamlExpr<'a>>, update: Box<DamlExpr<'a>>, field: &'a str) -> Self {
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
        self.field
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

#[derive(Debug, Serialize, Clone)]
pub struct DamlVariantCon<'a> {
    tycon: DamlTyCon<'a>,
    variant_arg: Box<DamlExpr<'a>>,
    variant_con: &'a str,
}

impl<'a> DamlVariantCon<'a> {
    pub fn new(tycon: DamlTyCon<'a>, variant_arg: Box<DamlExpr<'a>>, variant_con: &'a str) -> Self {
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
        self.variant_con
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

#[derive(Debug, Serialize, Clone)]
pub struct DamlEnumCon<'a> {
    tycon: DamlTyConName<'a>,
    enum_con: &'a str,
}

impl<'a> DamlEnumCon<'a> {
    pub fn new(tycon: DamlTyConName<'a>, enum_con: &'a str) -> Self {
        Self {
            tycon,
            enum_con,
        }
    }

    pub fn tycon(&self) -> &DamlTyConName<'a> {
        &self.tycon
    }

    pub fn enum_con(&self) -> &str {
        self.enum_con
    }
}

impl<'a> DamlVisitableElement<'a> for DamlEnumCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_enum_con(self);
        self.tycon.accept(visitor);
        visitor.post_visit_enum_con(self);
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

#[derive(Debug, Serialize, Clone)]
pub struct DamlStructProj<'a> {
    struct_expr: Box<DamlExpr<'a>>,
    field: &'a str,
}

impl<'a> DamlStructProj<'a> {
    pub fn new(struct_expr: Box<DamlExpr<'a>>, field: &'a str) -> Self {
        Self {
            struct_expr,
            field,
        }
    }

    pub fn struct_expr(&self) -> &DamlExpr<'a> {
        self.struct_expr.as_ref()
    }

    pub fn field(&self) -> &str {
        self.field
    }
}

impl<'a> DamlVisitableElement<'a> for DamlStructProj<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_struct_proj(self);
        self.struct_expr.accept(visitor);
        visitor.post_visit_struct_proj(self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlStructUpd<'a> {
    struct_expr: Box<DamlExpr<'a>>,
    update: Box<DamlExpr<'a>>,
    field: &'a str,
}

impl<'a> DamlStructUpd<'a> {
    pub fn new(struct_expr: Box<DamlExpr<'a>>, update: Box<DamlExpr<'a>>, field: &'a str) -> Self {
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
        self.field
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

#[derive(Debug, Serialize, Clone)]
pub struct DamlVarWithType<'a> {
    ty: DamlType<'a>,
    var: &'a str,
}

impl<'a> DamlVarWithType<'a> {
    pub fn new(ty: DamlType<'a>, var: &'a str) -> Self {
        Self {
            ty,
            var,
        }
    }

    pub fn ty(&self) -> &DamlType<'a> {
        &self.ty
    }

    pub fn var(&self) -> &str {
        self.var
    }
}

impl<'a> DamlVisitableElement<'a> for DamlVarWithType<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_var_with_type(self);
        self.ty.accept(visitor);
        visitor.post_visit_var_with_type(self);
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

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
pub struct DamlCaseAltVariant<'a> {
    con: DamlTyConName<'a>,
    variant: &'a str,
    binder: &'a str,
}

impl<'a> DamlCaseAltVariant<'a> {
    pub fn new(con: DamlTyConName<'a>, variant: &'a str, binder: &'a str) -> Self {
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
        self.variant
    }

    pub fn binder(&self) -> &str {
        self.binder
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCaseAltVariant<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_case_alt_variant(self);
        self.con.accept(visitor);
        visitor.post_visit_case_alt_variant(self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlCaseAltCons<'a> {
    var_head: &'a str,
    var_tail: &'a str,
}

impl<'a> DamlCaseAltCons<'a> {
    pub fn new(var_head: &'a str, var_tail: &'a str) -> Self {
        Self {
            var_head,
            var_tail,
        }
    }

    pub fn var_head(&self) -> &str {
        self.var_head
    }

    pub fn var_tail(&self) -> &str {
        self.var_tail
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCaseAltCons<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_case_alt_cons(self);
        visitor.post_visit_case_alt_cons(self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlCaseAltOptionalSome<'a> {
    var_body: &'a str,
}

impl<'a> DamlCaseAltOptionalSome<'a> {
    pub fn new(var_body: &'a str) -> Self {
        Self {
            var_body,
        }
    }

    pub fn var_body(&self) -> &str {
        self.var_body
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCaseAltOptionalSome<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_case_alt_opt_some(self);
        visitor.post_visit_case_alt_opt_some(self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlCaseAltEnum<'a> {
    con: DamlTyConName<'a>,
    constructor: &'a str,
}

impl<'a> DamlCaseAltEnum<'a> {
    pub fn new(con: DamlTyConName<'a>, constructor: &'a str) -> Self {
        Self {
            con,
            constructor,
        }
    }

    pub fn con(&self) -> &DamlTyConName<'a> {
        &self.con
    }

    pub fn constructor(&self) -> &str {
        self.constructor
    }
}

impl<'a> DamlVisitableElement<'a> for DamlCaseAltEnum<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_case_alt_enum(self);
        self.con.accept(visitor);
        visitor.post_visit_case_alt_enum(self);
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

#[derive(Debug, Serialize, Clone)]
pub enum DamlUpdate<'a> {
    Pure(DamlPure<'a>),
    Block(DamlBlock<'a>),
    Create(DamlCreate<'a>),
    Exercise(DamlExercise<'a>),
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
            DamlUpdate::Fetch(fetch) => fetch.accept(visitor),
            DamlUpdate::LookupByKey(retrieve_by_key) | DamlUpdate::FetchByKey(retrieve_by_key) =>
                retrieve_by_key.accept(visitor),
            DamlUpdate::EmbedExpr(embed_expr) => embed_expr.accept(visitor),
            DamlUpdate::GetTime => {},
        }
        visitor.post_visit_update(self);
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

#[derive(Debug, Serialize, Clone)]
pub struct DamlCreate<'a> {
    template: DamlTyConName<'a>,
    expr: Box<DamlExpr<'a>>,
}

impl<'a> DamlCreate<'a> {
    pub fn new(template: DamlTyConName<'a>, expr: Box<DamlExpr<'a>>) -> Self {
        Self {
            expr,
            template,
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

#[derive(Debug, Serialize, Clone)]
pub struct DamlExercise<'a> {
    template: DamlTyConName<'a>,
    cid: Box<DamlExpr<'a>>,
    actor: Option<Box<DamlExpr<'a>>>,
    arg: Box<DamlExpr<'a>>,
    choice: &'a str,
}

impl<'a> DamlExercise<'a> {
    pub fn new(
        template: DamlTyConName<'a>,
        cid: Box<DamlExpr<'a>>,
        actor: Option<Box<DamlExpr<'a>>>,
        arg: Box<DamlExpr<'a>>,
        choice: &'a str,
    ) -> Self {
        Self {
            template,
            cid,
            actor,
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

    pub fn actor(&self) -> Option<&DamlExpr<'a>> {
        self.actor.as_ref().map(AsRef::as_ref)
    }

    pub fn arg(&self) -> &DamlExpr<'a> {
        self.arg.as_ref()
    }

    pub fn choice(&self) -> &str {
        self.choice
    }
}

impl<'a> DamlVisitableElement<'a> for DamlExercise<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_exercise(self);
        self.template.accept(visitor);
        self.cid.accept(visitor);
        self.actor.as_ref().iter().for_each(|act| act.accept(visitor));
        self.arg.accept(visitor);
        visitor.post_visit_exercise(self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlFetch<'a> {
    template: DamlTyConName<'a>,
    cid: Box<DamlExpr<'a>>,
}

impl<'a> DamlFetch<'a> {
    pub fn new(template: DamlTyConName<'a>, cid: Box<DamlExpr<'a>>) -> Self {
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

#[derive(Debug, Serialize, Clone)]
pub struct DamlRetrieveByKey<'a> {
    template: DamlTyConName<'a>,
    key: Box<DamlExpr<'a>>,
}

impl<'a> DamlRetrieveByKey<'a> {
    pub fn new(template: DamlTyConName<'a>, key: Box<DamlExpr<'a>>) -> Self {
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

#[derive(Debug, Serialize, Clone)]
pub struct DamlDefKey<'a> {
    pub ty: DamlType<'a>,
    pub maintainers: DamlExpr<'a>,
    pub key_expr: DamlExpr<'a>,
}

impl<'a> DamlDefKey<'a> {
    pub fn new(ty: DamlType<'a>, maintainers: DamlExpr<'a>, key_expr: DamlExpr<'a>) -> Self {
        Self {
            ty,
            maintainers,
            key_expr,
        }
    }

    pub fn ty(&self) -> &DamlType<'a> {
        &self.ty
    }

    pub fn maintainers(&self) -> &DamlExpr<'a> {
        &self.maintainers
    }

    pub fn key_expr(&self) -> &DamlExpr<'a> {
        &self.key_expr
    }
}

impl<'a> DamlVisitableElement<'a> for DamlDefKey<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_def_key(self);
        self.ty.accept(visitor);
        self.maintainers.accept(visitor);
        self.key_expr.accept(visitor);
        visitor.post_visit_def_key(self);
    }
}
