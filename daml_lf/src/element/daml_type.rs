use crate::element::visitor::DamlElementVisitor;
use crate::element::DamlVisitableElement;
use serde::Serialize;

/// Representation of a DAML type.
#[derive(Debug, Serialize, Clone)]
pub enum DamlType<'a> {
    ContractId(Option<DamlDataRef<'a>>),
    Int64,
    Numeric(Box<DamlType<'a>>),
    Text,
    Timestamp,
    Party,
    Bool,
    Unit,
    Date,
    List(Box<DamlType<'a>>),
    TextMap(Box<DamlType<'a>>),
    Optional(Box<DamlType<'a>>),
    DataRef(DamlDataRef<'a>),
    BoxedDataRef(DamlDataRef<'a>),
    Var(DamlVar<'a>),
    Nat(u8),
    Arrow,
    Any,
    TypeRep,
    Update,
    Scenario,
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
            DamlType::DataRef(_) => "None (DataRef)",
            DamlType::BoxedDataRef(_) => "None (BoxedDataRef)",
            DamlType::Var(_) => "None (Var)",
            DamlType::Arrow => "None (Arrow)",
            DamlType::Any => "None (Any)",
            DamlType::TypeRep => "None (TypeRep)",
            DamlType::Nat(_) => "Nat",
        }
    }

    /// Returns true if this [`DamlType`] contain a reference to `type_var`, false otherwise.
    pub fn contains_type_var(&self, type_var: &str) -> bool {
        fn data_ref_contains_type_var(data_ref: &DamlDataRef<'_>, type_var: &str) -> bool {
            match data_ref {
                DamlDataRef::Local(local) => local.type_arguments.iter().any(|f| f.contains_type_var(type_var)),
                DamlDataRef::NonLocal(non_local) =>
                    non_local.type_arguments.iter().any(|f| f.contains_type_var(type_var)),
                DamlDataRef::Absolute(abs) => abs.type_arguments.iter().any(|f| f.contains_type_var(type_var)),
            }
        }
        match self {
            &DamlType::Var(DamlVar {
                var,
                ..
            }) => var == type_var,
            DamlType::List(inner) | DamlType::TextMap(inner) | DamlType::Optional(inner) | DamlType::Numeric(inner) =>
                inner.contains_type_var(type_var),
            DamlType::ContractId(data_ref) =>
                data_ref.as_ref().map_or(false, |dr| data_ref_contains_type_var(dr, type_var)),
            DamlType::DataRef(data_ref) | DamlType::BoxedDataRef(data_ref) =>
                data_ref_contains_type_var(data_ref, type_var),
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
            DamlType::List(inner) | DamlType::TextMap(inner) | DamlType::Optional(inner) | DamlType::Numeric(inner) =>
                inner.accept(visitor),
            DamlType::ContractId(data_ref) => data_ref.as_ref().map_or_else(|| {}, |dr| dr.accept(visitor)),
            DamlType::DataRef(data_ref) | DamlType::BoxedDataRef(data_ref) => data_ref.accept(visitor),
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

#[derive(Debug, Serialize, Clone)]
pub enum DamlDataRef<'a> {
    Local(DamlLocalDataRef<'a>),
    NonLocal(DamlNonLocalDataRef<'a>),
    Absolute(DamlAbsoluteDataRef<'a>),
}

impl<'a> DamlDataRef<'a> {
    pub fn reference_parts(&self) -> (&str, &[&str], &str) {
        match self {
            DamlDataRef::Local(local) => (local.package_name, &local.module_path, local.data_name),
            DamlDataRef::NonLocal(non_local) =>
                (non_local.target_package_name, &non_local.target_module_path, non_local.data_name),
            DamlDataRef::Absolute(abs) => (abs.package_name, &abs.module_path, abs.data_name),
        }
    }

    pub fn type_arguments(&self) -> &[DamlType<'_>] {
        match self {
            DamlDataRef::Local(local) => &local.type_arguments,
            DamlDataRef::NonLocal(non_local) => &non_local.type_arguments,
            DamlDataRef::Absolute(abs) => &abs.type_arguments,
        }
    }
}

impl<'a> DamlVisitableElement<'a> for DamlDataRef<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_data_ref(self);
        match self {
            DamlDataRef::Local(local) => local.accept(visitor),
            DamlDataRef::NonLocal(non_local) => non_local.accept(visitor),
            DamlDataRef::Absolute(absolute) => absolute.accept(visitor),
        }
        visitor.post_visit_data_ref(self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlLocalDataRef<'a> {
    data_name: &'a str,
    package_name: &'a str,
    module_path: Vec<&'a str>,
    type_arguments: Vec<DamlType<'a>>,
}

impl<'a> DamlLocalDataRef<'a> {
    pub fn new(
        data_name: &'a str,
        package_name: &'a str,
        module_path: Vec<&'a str>,
        type_arguments: Vec<DamlType<'a>>,
    ) -> Self {
        Self {
            data_name,
            package_name,
            module_path,
            type_arguments,
        }
    }

    pub const fn data_name(&self) -> &str {
        self.data_name
    }

    pub const fn package_name(&self) -> &str {
        self.package_name
    }

    pub fn module_path(&self) -> &[&str] {
        &self.module_path
    }

    pub fn type_arguments(&self) -> &[DamlType<'a>] {
        &self.type_arguments
    }
}

impl<'a> DamlVisitableElement<'a> for DamlLocalDataRef<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_local_data_ref(self);
        self.type_arguments.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_local_data_ref(self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlNonLocalDataRef<'a> {
    data_name: &'a str,
    source_package_name: &'a str,
    source_module_path: Vec<&'a str>,
    target_package_name: &'a str,
    target_module_path: Vec<&'a str>,
    type_arguments: Vec<DamlType<'a>>,
}

impl<'a> DamlNonLocalDataRef<'a> {
    pub fn new(
        data_name: &'a str,
        source_package_name: &'a str,
        source_module_path: Vec<&'a str>,
        target_package_name: &'a str,
        target_module_path: Vec<&'a str>,
        type_arguments: Vec<DamlType<'a>>,
    ) -> Self {
        Self {
            data_name,
            source_package_name,
            source_module_path,
            target_package_name,
            target_module_path,
            type_arguments,
        }
    }

    pub const fn data_name(&self) -> &str {
        self.data_name
    }

    pub const fn source_package_name(&self) -> &str {
        self.source_package_name
    }

    pub fn source_module_path(&self) -> &[&str] {
        &self.source_module_path
    }

    pub const fn target_package_name(&self) -> &str {
        self.target_package_name
    }

    pub fn target_module_path(&self) -> &[&str] {
        &self.target_module_path
    }

    pub fn type_arguments(&self) -> &[DamlType<'a>] {
        &self.type_arguments
    }
}

impl<'a> DamlVisitableElement<'a> for DamlNonLocalDataRef<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_non_local_data_ref(self);
        self.type_arguments.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_non_local_data_ref(self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlAbsoluteDataRef<'a> {
    data_name: &'a str,
    package_name: &'a str,
    module_path: Vec<&'a str>,
    type_arguments: Vec<DamlType<'a>>,
}

impl<'a> DamlAbsoluteDataRef<'a> {
    pub fn new(
        data_name: &'a str,
        package_name: &'a str,
        module_path: Vec<&'a str>,
        type_arguments: Vec<DamlType<'a>>,
    ) -> Self {
        Self {
            data_name,
            package_name,
            module_path,
            type_arguments,
        }
    }

    pub const fn data_name(&self) -> &str {
        self.data_name
    }

    pub const fn package_name(&self) -> &str {
        self.package_name
    }

    pub fn module_path(&self) -> &[&str] {
        &self.module_path
    }

    pub fn type_arguments(&self) -> &[DamlType<'a>] {
        &self.type_arguments
    }
}

impl<'a> DamlVisitableElement<'a> for DamlAbsoluteDataRef<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_absolute_data_ref(self);
        self.type_arguments.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_absolute_data_ref(self);
    }
}

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
