use crate::element::visitor::DamlElementVisitor;
use crate::element::{DamlField, DamlTypeVarWithKind, DamlVisitableElement};
use serde::Serialize;

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
            &DamlType::Var(DamlVar {
                var,
                ..
            }) => var == type_var,
            DamlType::Numeric(inner) => inner.contains_type_var(type_var),
            DamlType::List(args) | DamlType::Optional(args) | DamlType::TextMap(args) =>
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
}

impl<'a> DamlVisitableElement<'a> for DamlType<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_type(self);
        match self {
            DamlType::Var(var) => var.accept(visitor),
            DamlType::Numeric(inner) => inner.accept(visitor),
            DamlType::List(args) | DamlType::Optional(args) | DamlType::TextMap(args) =>
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

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub enum DamlTyConName<'a> {
    Local(DamlLocalTyCon<'a>),
    NonLocal(DamlNonLocalTyCon<'a>),
    Absolute(DamlAbsoluteTyCon<'a>),
}

impl<'a> DamlTyConName<'a> {
    pub fn reference_parts(&self) -> (&str, &[&str], &str) {
        match self {
            DamlTyConName::Local(local) => (local.package_name, &local.module_path, local.data_name),
            DamlTyConName::NonLocal(non_local) =>
                (non_local.target_package_name, &non_local.target_module_path, non_local.data_name),
            DamlTyConName::Absolute(abs) => (abs.package_name, &abs.module_path, abs.data_name),
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

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlLocalTyCon<'a> {
    data_name: &'a str,
    package_id: &'a str,
    package_name: &'a str,
    module_path: Vec<&'a str>,
}

impl<'a> DamlLocalTyCon<'a> {
    pub fn new(data_name: &'a str, package_id: &'a str, package_name: &'a str, module_path: Vec<&'a str>) -> Self {
        Self {
            data_name,
            package_id,
            package_name,
            module_path,
        }
    }

    pub const fn data_name(&self) -> &str {
        self.data_name
    }

    pub const fn package_id(&self) -> &str {
        self.package_id
    }

    pub const fn package_name(&self) -> &str {
        self.package_name
    }

    pub fn module_path(&self) -> &[&str] {
        &self.module_path
    }
}

impl<'a> DamlVisitableElement<'a> for DamlLocalTyCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_local_tycon(self);
        visitor.post_visit_local_tycon(self);
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlNonLocalTyCon<'a> {
    data_name: &'a str,
    source_package_id: &'a str,
    source_package_name: &'a str,
    source_module_path: Vec<&'a str>,
    target_package_id: &'a str,
    target_package_name: &'a str,
    target_module_path: Vec<&'a str>,
}

impl<'a> DamlNonLocalTyCon<'a> {
    pub fn new(
        data_name: &'a str,
        source_package_id: &'a str,
        source_package_name: &'a str,
        source_module_path: Vec<&'a str>,
        target_package_id: &'a str,
        target_package_name: &'a str,
        target_module_path: Vec<&'a str>,
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

    pub const fn data_name(&self) -> &str {
        self.data_name
    }

    pub const fn source_package_id(&self) -> &str {
        self.source_package_id
    }

    pub const fn source_package_name(&self) -> &str {
        self.source_package_name
    }

    pub fn source_module_path(&self) -> &[&str] {
        &self.source_module_path
    }

    pub const fn target_package_id(&self) -> &str {
        self.target_package_id
    }

    pub const fn target_package_name(&self) -> &str {
        self.target_package_name
    }

    pub fn target_module_path(&self) -> &[&str] {
        &self.target_module_path
    }
}

impl<'a> DamlVisitableElement<'a> for DamlNonLocalTyCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_non_local_tycon(self);
        visitor.post_visit_non_local_tycon(self);
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlAbsoluteTyCon<'a> {
    data_name: &'a str,
    package_id: &'a str,
    package_name: &'a str,
    module_path: Vec<&'a str>,
}

impl<'a> DamlAbsoluteTyCon<'a> {
    pub fn new(data_name: &'a str, package_id: &'a str, package_name: &'a str, module_path: Vec<&'a str>) -> Self {
        Self {
            data_name,
            package_id,
            package_name,
            module_path,
        }
    }

    pub const fn data_name(&self) -> &str {
        self.data_name
    }

    pub const fn package_id(&self) -> &str {
        self.package_id
    }

    pub const fn package_name(&self) -> &str {
        self.package_name
    }

    pub fn module_path(&self) -> &[&str] {
        &self.module_path
    }
}

impl<'a> DamlVisitableElement<'a> for DamlAbsoluteTyCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_absolute_tycon(self);
        visitor.post_visit_absolute_tycon(self);
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlVar<'a> {
    var: &'a str,
    type_arguments: Vec<DamlType<'a>>,
}

impl<'a> DamlVar<'a> {
    pub fn new(var: &'a str, type_arguments: Vec<DamlType<'a>>) -> Self {
        Self {
            var,
            type_arguments,
        }
    }

    pub const fn var(&self) -> &str {
        self.var
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
