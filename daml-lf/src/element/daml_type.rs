use std::borrow::Cow;

use serde::Serialize;

use crate::element::visitor::DamlElementVisitor;
use crate::element::{DamlData, DamlField, DamlTypeVarWithKind, DamlVisitableElement};
use bounded_static::ToStatic;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

/// A Daml type.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub enum DamlType<'a> {
    /// Opaque contract identifier.
    ContractId(Option<Box<DamlType<'a>>>),
    /// Signed 64 bit integer.
    Int64,
    /// Fixed precision numeric.
    Numeric(Vec<DamlType<'a>>),
    /// Unicode text data.
    Text,
    /// A date & time.
    Timestamp,
    /// A Daml Party.
    Party,
    /// A Boolean type.
    Bool,
    /// A Unit type.
    Unit,
    /// A date.
    Date,
    /// A list.
    List(Vec<DamlType<'a>>),
    /// A map wih [DamlType::Text] keys.
    TextMap(Vec<DamlType<'a>>),
    /// A map.
    GenMap(Vec<DamlType<'a>>),
    /// An optional value.
    Optional(Vec<DamlType<'a>>),
    /// A type constructor.
    TyCon(DamlTyCon<'a>),
    /// A type constructor (heap allocated).
    BoxedTyCon(DamlTyCon<'a>),
    /// A type variable.
    Var(DamlVar<'a>),
    /// A natural number.
    Nat(u8),
    /// A function.
    Arrow,
    /// Any type.
    Any,
    /// A type rep.
    TypeRep,
    /// A big numeric.
    Bignumeric,
    /// A rounding mode.
    RoundingMode,
    /// An exception.
    AnyException,
    /// An update effect.
    Update,
    /// A scenario effect.
    Scenario,
    /// Universal qualifier.
    Forall(DamlForall<'a>),
    /// A struct type.
    Struct(DamlStruct<'a>),
    /// A type synonym.
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
            DamlType::RoundingMode => "None (RoundingMode)",
            DamlType::AnyException => "None (AnyException)",
            DamlType::Bignumeric => "None (Bignumeric)",
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
            DamlType::List(args)
            | DamlType::Optional(args)
            | DamlType::TextMap(args)
            | DamlType::GenMap(args)
            | DamlType::Numeric(args) => args.iter().any(|arg| arg.contains_type_var(type_var)),
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
            | DamlType::Bignumeric
            | DamlType::RoundingMode
            | DamlType::AnyException
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
        DamlType::TyCon(DamlTyCon::new_absolute_with_type_args(package_id, module, entity, type_arguments))
    }
}

impl<'a> DamlVisitableElement<'a> for DamlType<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_type(self);
        match self {
            DamlType::Var(var) => var.accept(visitor),
            DamlType::List(args)
            | DamlType::Optional(args)
            | DamlType::TextMap(args)
            | DamlType::GenMap(args)
            | DamlType::Numeric(args) => args.iter().for_each(|arg| arg.accept(visitor)),
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
            | DamlType::Bignumeric
            | DamlType::RoundingMode
            | DamlType::AnyException
            | DamlType::Nat(_) => {},
        }
        visitor.post_visit_type(self);
    }
}

/// `DamlTypeSynName` is aliases from `DamlTypeConName` as they are currently identical.
pub type DamlTypeSynName<'a> = DamlTyConName<'a>;

/// A Daml type synonym.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlSyn<'a> {
    pub tysyn: Box<DamlTypeSynName<'a>>,
    pub args: Vec<DamlType<'a>>,
}

impl<'a> DamlSyn<'a> {
    pub fn new(tysyn: Box<DamlTypeSynName<'a>>, args: Vec<DamlType<'a>>) -> Self {
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

/// A Daml struct.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// Universal qualifier.
#[derive(Debug, Serialize, Clone, ToStatic)]
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

/// A Daml type constructor.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlTyCon<'a> {
    tycon: Box<DamlTyConName<'a>>,
    type_arguments: Vec<DamlType<'a>>,
}

impl<'a> DamlTyCon<'a> {
    pub fn new(tycon: Box<DamlTyConName<'a>>, type_arguments: Vec<DamlType<'a>>) -> Self {
        Self {
            tycon,
            type_arguments,
        }
    }

    pub fn new_absolute<'b, S: AsRef<str> + 'b>(
        package_id: &'b str,
        module: &'b [S],
        entity: &'b str,
    ) -> DamlTyCon<'b> {
        Self::new_absolute_with_type_args(package_id, module, entity, vec![])
    }

    pub fn new_absolute_with_type_args<'b, S: AsRef<str> + 'b>(
        package_id: &'b str,
        module: &'b [S],
        entity: &'b str,
        type_arguments: Vec<DamlType<'b>>,
    ) -> DamlTyCon<'b> {
        DamlTyCon::new(Box::new(DamlTyConName::new_absolute(package_id, module, entity)), type_arguments)
    }

    pub fn type_arguments(&self) -> &[DamlType<'_>] {
        &self.type_arguments
    }

    pub fn tycon(&self) -> &DamlTyConName<'_> {
        &self.tycon
    }
}

impl Hash for DamlTyCon<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tycon.hash(state);
    }
}

/// Equality for `DamlTyCon` is defined on the `DamlTyConName` only, `type_arguments` are not considered.
impl PartialEq for DamlTyCon<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.tycon == other.tycon
    }
}

impl Eq for DamlTyCon<'_> {}

impl<'a> DamlVisitableElement<'a> for DamlTyCon<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_tycon(self);
        self.tycon.accept(visitor);
        self.type_arguments.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_tycon(self);
    }
}

/// A Daml type constructor.
#[derive(Debug, Serialize, Clone, Hash, Eq, PartialEq, ToStatic)]
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

    pub fn new_absolute<'b, S: AsRef<str> + 'b>(
        package_id: &'b str,
        module: &'b [S],
        entity: &'b str,
    ) -> DamlTyConName<'b> {
        DamlTyConName::Absolute(DamlAbsoluteTyCon::new(
            entity.into(),
            package_id.into(),
            Cow::default(),
            module.iter().map(AsRef::as_ref).map(Into::into).collect(),
        ))
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
            DamlTyConName::Local(local) => {
                write!(f, "{}:{}:{}", local.package_name, &local.module_path.join("."), local.data_name)
            },
            DamlTyConName::NonLocal(non_local) => write!(
                f,
                "{}:{}:{}",
                non_local.target_package_name,
                &non_local.target_module_path.join("."),
                non_local.data_name
            ),
            DamlTyConName::Absolute(abs) => {
                write!(f, "{}:{}:{}", abs.package_name, &abs.module_path.join("."), abs.data_name)
            },
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

/// Convenience impl to compare a `DamlData` with a `DamlTyConName`.
impl PartialEq<DamlData<'_>> for DamlTyConName<'_> {
    fn eq(&self, data: &DamlData<'_>) -> bool {
        data == self
    }
}

/// A Daml local type constructor.
#[derive(Debug, Serialize, Clone, Hash, Eq, PartialEq, ToStatic)]
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

/// A Daml non-local type constructor.
#[derive(Debug, Serialize, Clone, Hash, Eq, PartialEq, ToStatic)]
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

/// A Daml absolute type constructor.
#[derive(Debug, Serialize, Clone, Hash, Eq, PartialEq, ToStatic)]
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

/// A Daml type variable.
#[derive(Debug, Serialize, Clone, ToStatic)]
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
