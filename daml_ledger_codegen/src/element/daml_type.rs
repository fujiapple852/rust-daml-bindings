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
    Var,
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
            _ => panic!(format!("DamlType::name called for unsupported type {:?}", self)),
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
}

impl<'a> DamlLocalDataRef<'a> {
    pub fn new(data_name: &'a str, package_name: &'a str, module_path: Vec<&'a str>) -> Self {
        Self {
            data_name,
            package_name,
            module_path,
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
}

impl<'a> DamlNonLocalDataRef<'a> {
    pub fn new(
        data_name: &'a str,
        source_package_name: &'a str,
        source_module_path: Vec<&'a str>,
        target_package_name: &'a str,
        target_module_path: Vec<&'a str>,
    ) -> Self {
        Self {
            data_name,
            source_package_name,
            source_module_path,
            target_package_name,
            target_module_path,
        }
    }
}

#[derive(Debug)]
pub struct DamlAbsoluteDataRef<'a> {
    pub data_name: &'a str,
    pub package_name: &'a str,
    pub module_path: Vec<&'a str>,
}

impl<'a> DamlAbsoluteDataRef<'a> {
    pub fn new(data_name: &'a str, package_name: &'a str, module_path: Vec<&'a str>) -> Self {
        Self {
            data_name,
            package_name,
            module_path,
        }
    }
}
