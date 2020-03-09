// TODO support GenMap
/// Representation of a DAML type.
#[derive(Debug)]
pub enum DamlType<'a> {
    ContractId(Option<DamlDataRef<'a>>),
    Int64,
    Numeric,
    Text,
    Timestamp,
    Party,
    Bool,
    Unit,
    Date,
    List(Box<DamlType<'a>>),
    Update,
    Scenario,
    TextMap(Box<DamlType<'a>>),
    Optional(Box<DamlType<'a>>),
    DataRef(DamlDataRef<'a>),
    BoxedDataRef(DamlDataRef<'a>),
    Var(DamlVar<'a>),
    Arrow,
    Any,
    TypeRep,
}

impl<'a> DamlType<'a> {
    pub fn name(&self) -> &str {
        match self {
            DamlType::ContractId(_) => "DamlContractId",
            DamlType::Int64 => "DamlInt64",
            DamlType::Numeric => "DamlNumeric",
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
            DamlType::List(inner) | DamlType::TextMap(inner) | DamlType::Optional(inner) =>
                inner.contains_type_var(type_var),
            DamlType::ContractId(data_ref) =>
                data_ref.as_ref().map_or(false, |dr| data_ref_contains_type_var(dr, type_var)),
            DamlType::DataRef(data_ref) | DamlType::BoxedDataRef(data_ref) =>
                data_ref_contains_type_var(data_ref, type_var),
            DamlType::Int64
            | DamlType::Numeric
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
            | DamlType::TypeRep => false,
        }
    }
}

#[derive(Debug)]
pub enum DamlDataRef<'a> {
    Local(DamlLocalDataRef<'a>),
    NonLocal(DamlNonLocalDataRef<'a>),
    Absolute(DamlAbsoluteDataRef<'a>),
}

#[derive(Debug)]
pub struct DamlLocalDataRef<'a> {
    pub data_name: &'a str,
    pub package_name: &'a str,
    pub module_path: Vec<&'a str>,
    pub type_arguments: Vec<DamlType<'a>>,
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
}

#[derive(Debug)]
pub struct DamlNonLocalDataRef<'a> {
    pub data_name: &'a str,
    pub source_package_name: &'a str,
    pub source_module_path: Vec<&'a str>,
    pub target_package_name: &'a str,
    pub target_module_path: Vec<&'a str>,
    pub type_arguments: Vec<DamlType<'a>>,
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
}

#[derive(Debug)]
pub struct DamlAbsoluteDataRef<'a> {
    pub data_name: &'a str,
    pub package_name: &'a str,
    pub module_path: Vec<&'a str>,
    pub type_arguments: Vec<DamlType<'a>>,
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
}

#[derive(Debug)]
pub struct DamlVar<'a> {
    pub var: &'a str,
    pub type_arguments: Vec<DamlType<'a>>,
}

impl<'a> DamlVar<'a> {
    pub fn new(var: &'a str, type_arguments: Vec<DamlType<'a>>) -> Self {
        Self {
            var,
            type_arguments,
        }
    }
}
