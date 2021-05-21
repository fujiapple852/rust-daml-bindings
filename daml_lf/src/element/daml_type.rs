use std::borrow::Cow;

use serde::Serialize;

use crate::element::visitor::DamlElementVisitor;
use crate::element::{DamlField, DamlTypeVarWithKind, DamlVisitableElement};
use crate::owned::ToStatic;
use std::fmt;
use std::fmt::{Display, Formatter};

/// Representation of a DAML type.
#[derive(Debug, Serialize, Clone)]
pub enum DamlType<'a> {
    ContractId(Option<Box<DamlType<'a>>>),
    Int64,
    Numeric(Box<DamlType<'a>>),
    Text,
    Timestamp,
    Party,
    Bool,
    Unit,
    Date,
    List(Vec<DamlType<'a>>),
    TextMap(Vec<DamlType<'a>>),
    GenMap(Vec<DamlType<'a>>),
    Optional(Vec<DamlType<'a>>),
    TyCon(DamlTyCon<'a>),
    BoxedTyCon(DamlTyCon<'a>),
    Var(DamlVar<'a>),
    Nat(u8),
    Arrow,
    Any,
    TypeRep,
    Update,
    Scenario,
    Forall(DamlForall<'a>),
    Struct(DamlStruct<'a>),
    Syn(DamlSyn<'a>),
}

impl<'a> DamlType<'a> {
    pub fn name(&self) -> &str {
        match self {
            DamlType::ContractId(_) => "DamlContractId",
            DamlType::Int64 => "DamlInt64",
            DamlType::Numeric(_) => "DamlFixedNumeric",
            DamlType::Text => "DamlText",
            DamlType::Timestamp => "DamlTimestamp",
            DamlType::Party => "DamlParty",
            DamlType::Bool => "DamlBool",
            DamlType::Unit => "DamlUnit",
            DamlType::Date => "DamlDate",
            DamlType::List(_) => "DamlList",
            DamlType::TextMap(_) => "DamlTextMap",
            DamlType::GenMap(_) => "DamlGenMap",
            DamlType::Optional(_) => "DamlOptional",
            DamlType::Update => "None (Update)",
            DamlType::Scenario => "None (Scenario)",
            DamlType::TyCon(_) => "None (TyCon)",
            DamlType::BoxedTyCon(_) => "None (BoxedTyCon)",
            DamlType::Var(_) => "None (Var)",
            DamlType::Arrow => "None (Arrow)",
            DamlType::Any => "None (Any)",
            DamlType::TypeRep => "None (TypeRep)",
            DamlType::Nat(_) => "Nat",
            DamlType::Forall(_) => "Forall",
            DamlType::Struct(_) => "Struct",
            DamlType::Syn(_) => "Syn",
        }
    }

    /// Returns true if this [`DamlType`] contain a reference to `type_var`, false otherwise.
    pub fn contains_type_var(&self, type_var: &str) -> bool {
        match self {
            DamlType::Var(DamlVar {
                var,
                ..
            }) => var == type_var,
            DamlType::Numeric(inner) => inner.contains_type_var(type_var),
            DamlType::List(args) | DamlType::Optional(args) | DamlType::TextMap(args) | DamlType::GenMap(args) =>
                args.iter().any(|arg| arg.contains_type_var(type_var)),
            DamlType::ContractId(inner) => inner.as_ref().map_or(false, |ty| ty.contains_type_var(type_var)),
            DamlType::TyCon(tycon) | DamlType::BoxedTyCon(tycon) =>
                tycon.type_arguments.iter().any(|f| f.contains_type_var(type_var)),
            DamlType::Forall(forall) => forall.body.as_ref().contains_type_var(type_var),
            DamlType::Struct(tuple) => tuple.fields.iter().any(|field| field.ty().contains_type_var(type_var)),
            DamlType::Syn(syn) => syn.args.iter().any(|arg| arg.contains_type_var(type_var)),
            DamlType::Int64
            | DamlType::Text
            | DamlType::Timestamp
            | DamlType::Party
            | DamlType::Bool
            | DamlType::Unit
            | DamlType::Date
            | DamlType::Update
            | DamlType::Scenario
            | DamlType::Arrow
            | DamlType::Any
            | DamlType::TypeRep
            | DamlType::Nat(_) => false,
        }
    }

    /// Convenience method to create a `DamlType::TyCon` for a given package id, module path and entity.
    pub fn make_tycon<'b, S: AsRef<str> + 'b>(package_id: &'b str, module: &'b [S], entity: &'b str) -> DamlType<'b> {
        Self::make_tycon_with_args(package_id, module, entity, vec![])
    }

    /// Convenience method to create a `DamlType::TyCon` for a given package id, module path, entity & type args.
    pub fn make_tycon_with_args<'b, S: AsRef<str> + 'b>(
        package_id: &'b str,
        module: &'b [S],
        entity: &'b str,
        type_arguments: Vec<DamlType<'b>>,
    ) -> DamlType<'b> {
        DamlType::TyCon(DamlTyCon::new(
            DamlTyConName::Absolute(DamlAbsoluteTyCon::new(
                entity.into(),
                package_id.into(),
                Cow::default(),
                module.iter().map(AsRef::as_ref).map(Into::into).collect(),
            )),
            type_arguments,
        ))
    }
}

impl<'a> DamlVisitableElement<'a> for DamlType<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_type(self);
        match self {
            DamlType::Var(var) => var.accept(visitor),
            DamlType::Numeric(inner) => inner.accept(visitor),
            DamlType::List(args) | DamlType::Optional(args) | DamlType::TextMap(args) | DamlType::GenMap(args) =>
                args.iter().for_each(|arg| arg.accept(visitor)),
            DamlType::ContractId(tycon) => tycon.as_ref().map_or_else(|| {}, |dr| dr.accept(visitor)),
            DamlType::TyCon(tycon) | DamlType::BoxedTyCon(tycon) => tycon.accept(visitor),
            DamlType::Forall(forall) => forall.accept(visitor),
            DamlType::Struct(tuple) => tuple.accept(visitor),
            DamlType::Syn(syn) => syn.accept(visitor),
            DamlType::Int64
            | DamlType::Text
            | DamlType::Timestamp
            | DamlType::Party
            | DamlType::Bool
            | DamlType::Unit
            | DamlType::Date
            | DamlType::Update
            | DamlType::Scenario
            | DamlType::Arrow
            | DamlType::Any
            | DamlType::TypeRep
            | DamlType::Nat(_) => {},
        }
        visitor.post_visit_type(self);
    }
}

impl ToStatic for DamlType<'_> {
    type Static = DamlType<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            DamlType::ContractId(inner) =>
                DamlType::ContractId(inner.as_ref().map(|ty| Box::new(DamlType::to_static(ty)))),
            DamlType::Int64 => DamlType::Int64,
            DamlType::Numeric(inner) => DamlType::Numeric(Box::new(inner.to_static())),
            DamlType::Text => DamlType::Text,
            DamlType::Timestamp => DamlType::Timestamp,
            DamlType::Party => DamlType::Party,
            DamlType::Bool => DamlType::Bool,
            DamlType::Unit => DamlType::Unit,
            DamlType::Date => DamlType::Date,
            DamlType::List(args) => DamlType::List(args.iter().map(DamlType::to_static).collect()),
            DamlType::TextMap(args) => DamlType::TextMap(args.iter().map(DamlType::to_static).collect()),
            DamlType::GenMap(args) => DamlType::GenMap(args.iter().map(DamlType::to_static).collect()),
            DamlType::Optional(args) => DamlType::Optional(args.iter().map(DamlType::to_static).collect()),
            DamlType::TyCon(tycon) => DamlType::TyCon(tycon.to_static()),
            DamlType::BoxedTyCon(tycon) => DamlType::BoxedTyCon(tycon.to_static()),
            DamlType::Var(var) => DamlType::Var(var.to_static()),
            DamlType::Nat(nat) => DamlType::Nat(*nat),
            DamlType::Arrow => DamlType::Arrow,
            DamlType::Any => DamlType::Any,
            DamlType::TypeRep => DamlType::TypeRep,
            DamlType::Update => DamlType::Update,
            DamlType::Scenario => DamlType::Scenario,
            DamlType::Forall(forall) => DamlType::Forall(forall.to_static()),
            DamlType::Struct(tuple) => DamlType::Struct(tuple.to_static()),
            DamlType::Syn(syn) => DamlType::Syn(syn.to_static()),
        }
    }
}

/// `DamlTypeSynName` is aliases from `DamlTypeConName` as they are currently identical.
pub type DamlTypeSynName<'a> = DamlTyConName<'a>;

#[derive(Debug, Serialize, Clone)]
pub struct DamlSyn<'a> {
    pub tysyn: DamlTypeSynName<'a>,
    pub args: Vec<DamlType<'a>>,
}

impl<'a> DamlSyn<'a> {
    pub fn new(tysyn: DamlTypeSynName<'a>, args: Vec<DamlType<'a>>) -> Self {
        Self {
            tysyn,
            args,
        }
    }

    pub fn tysyn(&self) -> &DamlTypeSynName<'a> {
        &self.tysyn
    }

    pub fn args(&self) -> &[DamlType<'_>] {
        &self.args
    }
}

impl<'a> DamlVisitableElement<'a> for DamlSyn<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_syn(self);
        self.tysyn.accept(visitor);
        self.args.iter().for_each(|field| field.accept(visitor));
        visitor.post_visit_syn(self);
    }
}

impl ToStatic for DamlSyn<'_> {
    type Static = DamlSyn<'static>;

    fn to_static(&self) -> Self::Static {
        DamlSyn::new(self.tysyn.to_static(), self.args.iter().map(DamlType::to_static).collect())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlStruct<'a> {
    pub fields: Vec<DamlField<'a>>,
}

impl<'a> DamlStruct<'a> {
    pub fn new(fields: Vec<DamlField<'a>>) -> Self {
        Self {
            fields,
        }
    }

    pub fn fields(&self) -> &[DamlField<'_>] {
        &self.fields
    }
}

impl<'a> DamlVisitableElement<'a> for DamlStruct<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_struct(self);
        self.fields.iter().for_each(|field| field.accept(visitor));
        visitor.post_visit_struct(self);
    }
}

impl ToStatic for DamlStruct<'_> {
    type Static = DamlStruct<'static>;

    fn to_static(&self) -> Self::Static {
        DamlStruct::new(self.fields.iter().map(DamlField::to_static).collect())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlForall<'a> {
    pub vars: Vec<DamlTypeVarWithKind<'a>>,
    pub body: Box<DamlType<'a>>,
}

impl<'a> DamlForall<'a> {
    pub fn new(vars: Vec<DamlTypeVarWithKind<'a>>, body: Box<DamlType<'a>>) -> Self {
        Self {
            vars,
            body,
        }
    }

    pub fn vars(&self) -> &[DamlTypeVarWithKind<'_>] {
        &self.vars
    }

    pub fn body(&self) -> &DamlType<'a> {
        &self.body
    }
}

impl<'a> DamlVisitableElement<'a> for DamlForall<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_forall(self);
        self.vars.iter().for_each(|var| var.accept(visitor));
        self.body.accept(visitor);
        visitor.post_visit_forall(self);
    }
}

impl ToStatic for DamlForall<'_> {
    type Static = DamlForall<'static>;

    fn to_static(&self) -> Self::Static {
        DamlForall::new(self.vars.iter().map(DamlTypeVarWithKind::to_static).collect(), Box::new(self.body.to_static()))
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlTyCon<'a> {
    tycon: DamlTyConName<'a>,
    type_arguments: Vec<DamlType<'a>>,
}

impl<'a> DamlTyCon<'a> {
    pub fn new(tycon: DamlTyConName<'a>, type_arguments: Vec<DamlType<'a>>) -> Self {
        Self {
            tycon,
            type_arguments,
        }
    }

    pub fn type_arguments(&self) -> &[DamlType<'_>] {
        &self.type_arguments
    }

    pub fn tycon(&self) -> &DamlTyConName<'_> {
        &self.tycon
    }
}

impl<'a> DamlVisitableElement<'a> for DamlTyCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_tycon(self);
        self.tycon.accept(visitor);
        self.type_arguments.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_tycon(self);
    }
}

impl ToStatic for DamlTyCon<'_> {
    type Static = DamlTyCon<'static>;

    fn to_static(&self) -> Self::Static {
        DamlTyCon::new(self.tycon.to_static(), self.type_arguments.iter().map(DamlType::to_static).collect())
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub enum DamlTyConName<'a> {
    Local(DamlLocalTyCon<'a>),
    NonLocal(DamlNonLocalTyCon<'a>),
    Absolute(DamlAbsoluteTyCon<'a>),
}

impl<'a> DamlTyConName<'a> {
    pub fn package_id(&self) -> &str {
        match self {
            DamlTyConName::Local(local) => &local.package_id,
            DamlTyConName::NonLocal(non_local) => &non_local.target_package_id,
            DamlTyConName::Absolute(abs) => &abs.package_id,
        }
    }

    pub fn package_name(&self) -> &str {
        match self {
            DamlTyConName::Local(local) => &local.package_name,
            DamlTyConName::NonLocal(non_local) => &non_local.target_package_name,
            DamlTyConName::Absolute(abs) => &abs.package_name,
        }
    }

    pub fn module_path(&self) -> impl Iterator<Item = &str> {
        match self {
            DamlTyConName::Local(local) => local.module_path.iter().map(AsRef::as_ref),
            DamlTyConName::NonLocal(non_local) => non_local.target_module_path.iter().map(AsRef::as_ref),
            DamlTyConName::Absolute(abs) => abs.module_path.iter().map(AsRef::as_ref),
        }
    }

    pub fn data_name(&self) -> &str {
        match self {
            DamlTyConName::Local(local) => &local.data_name,
            DamlTyConName::NonLocal(non_local) => &non_local.data_name,
            DamlTyConName::Absolute(abs) => &abs.data_name,
        }
    }

    /// Extract the package id, module path and data name.
    #[doc(hidden)]
    pub(crate) fn reference_parts(&self) -> (&str, &[Cow<'_, str>], &str) {
        match self {
            DamlTyConName::Local(local) => (&local.package_id, &local.module_path, &local.data_name),
            DamlTyConName::NonLocal(non_local) =>
                (&non_local.target_package_id, &non_local.target_module_path, &non_local.data_name),
            DamlTyConName::Absolute(abs) => (&abs.package_id, &abs.module_path, &abs.data_name),
        }
    }
}

impl Display for DamlTyConName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DamlTyConName::Local(local) =>
                write!(f, "{}:{}:{}", local.package_name, &local.module_path.join("."), local.data_name),
            DamlTyConName::NonLocal(non_local) => write!(
                f,
                "{}:{}:{}",
                non_local.target_package_name,
                &non_local.target_module_path.join("."),
                non_local.data_name
            ),
            DamlTyConName::Absolute(abs) =>
                write!(f, "{}:{}:{}", abs.package_name, &abs.module_path.join("."), abs.data_name),
        }
    }
}

impl<'a> DamlVisitableElement<'a> for DamlTyConName<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_tycon_name(self);
        match self {
            DamlTyConName::Local(local) => local.accept(visitor),
            DamlTyConName::NonLocal(non_local) => non_local.accept(visitor),
            DamlTyConName::Absolute(absolute) => absolute.accept(visitor),
        }
        visitor.post_visit_tycon_name(self);
    }
}

impl ToStatic for DamlTyConName<'_> {
    type Static = DamlTyConName<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            DamlTyConName::Local(local) => DamlTyConName::Local(local.to_static()),
            DamlTyConName::NonLocal(non_local) => DamlTyConName::NonLocal(non_local.to_static()),
            DamlTyConName::Absolute(absolute) => DamlTyConName::Absolute(absolute.to_static()),
        }
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlLocalTyCon<'a> {
    data_name: Cow<'a, str>,
    package_id: Cow<'a, str>,
    package_name: Cow<'a, str>,
    module_path: Vec<Cow<'a, str>>,
}

impl<'a> DamlLocalTyCon<'a> {
    pub fn new(
        data_name: Cow<'a, str>,
        package_id: Cow<'a, str>,
        package_name: Cow<'a, str>,
        module_path: Vec<Cow<'a, str>>,
    ) -> Self {
        Self {
            data_name,
            package_id,
            package_name,
            module_path,
        }
    }

    pub fn data_name(&self) -> &str {
        &self.data_name
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

impl<'a> DamlVisitableElement<'a> for DamlLocalTyCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_local_tycon(self);
        visitor.post_visit_local_tycon(self);
    }
}

impl ToStatic for DamlLocalTyCon<'_> {
    type Static = DamlLocalTyCon<'static>;

    fn to_static(&self) -> Self::Static {
        DamlLocalTyCon::new(
            self.data_name.to_static(),
            self.package_id.to_static(),
            self.package_name.to_static(),
            self.module_path.iter().map(ToStatic::to_static).collect(),
        )
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlNonLocalTyCon<'a> {
    data_name: Cow<'a, str>,
    source_package_id: Cow<'a, str>,
    source_package_name: Cow<'a, str>,
    source_module_path: Vec<Cow<'a, str>>,
    target_package_id: Cow<'a, str>,
    target_package_name: Cow<'a, str>,
    target_module_path: Vec<Cow<'a, str>>,
}

impl<'a> DamlNonLocalTyCon<'a> {
    pub fn new(
        data_name: Cow<'a, str>,
        source_package_id: Cow<'a, str>,
        source_package_name: Cow<'a, str>,
        source_module_path: Vec<Cow<'a, str>>,
        target_package_id: Cow<'a, str>,
        target_package_name: Cow<'a, str>,
        target_module_path: Vec<Cow<'a, str>>,
    ) -> Self {
        Self {
            data_name,
            source_package_id,
            source_package_name,
            source_module_path,
            target_package_id,
            target_package_name,
            target_module_path,
        }
    }

    pub fn data_name(&self) -> &str {
        &self.data_name
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

impl<'a> DamlVisitableElement<'a> for DamlNonLocalTyCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_non_local_tycon(self);
        visitor.post_visit_non_local_tycon(self);
    }
}

impl ToStatic for DamlNonLocalTyCon<'_> {
    type Static = DamlNonLocalTyCon<'static>;

    fn to_static(&self) -> Self::Static {
        DamlNonLocalTyCon::new(
            self.data_name.to_static(),
            self.source_package_id.to_static(),
            self.source_package_name.to_static(),
            self.source_module_path.iter().map(ToStatic::to_static).collect(),
            self.target_package_id.to_static(),
            self.target_package_name.to_static(),
            self.target_module_path.iter().map(ToStatic::to_static).collect(),
        )
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlAbsoluteTyCon<'a> {
    data_name: Cow<'a, str>,
    package_id: Cow<'a, str>,
    package_name: Cow<'a, str>,
    module_path: Vec<Cow<'a, str>>,
}

impl<'a> DamlAbsoluteTyCon<'a> {
    pub fn new(
        data_name: Cow<'a, str>,
        package_id: Cow<'a, str>,
        package_name: Cow<'a, str>,
        module_path: Vec<Cow<'a, str>>,
    ) -> Self {
        Self {
            data_name,
            package_id,
            package_name,
            module_path,
        }
    }

    pub fn data_name(&self) -> &str {
        &self.data_name
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

impl<'a> DamlVisitableElement<'a> for DamlAbsoluteTyCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_absolute_tycon(self);
        visitor.post_visit_absolute_tycon(self);
    }
}

impl ToStatic for DamlAbsoluteTyCon<'_> {
    type Static = DamlAbsoluteTyCon<'static>;

    fn to_static(&self) -> Self::Static {
        DamlAbsoluteTyCon::new(
            self.data_name.to_static(),
            self.package_id.to_static(),
            self.package_name.to_static(),
            self.module_path.iter().map(ToStatic::to_static).collect(),
        )
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlVar<'a> {
    var: Cow<'a, str>,
    type_arguments: Vec<DamlType<'a>>,
}

impl<'a> DamlVar<'a> {
    pub fn new(var: Cow<'a, str>, type_arguments: Vec<DamlType<'a>>) -> Self {
        Self {
            var,
            type_arguments,
        }
    }

    pub fn var(&self) -> &str {
        &self.var
    }

    pub fn type_arguments(&self) -> &[DamlType<'a>] {
        &self.type_arguments
    }
}

impl<'a> DamlVisitableElement<'a> for DamlVar<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_var(self);
        self.type_arguments.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_var(self);
    }
}

impl ToStatic for DamlVar<'_> {
    type Static = DamlVar<'static>;

    fn to_static(&self) -> Self::Static {
        DamlVar::new(self.var.to_static(), self.type_arguments.iter().map(DamlType::to_static).collect())
    }
}
