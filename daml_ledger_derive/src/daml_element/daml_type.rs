use itertools::Itertools;
use syn::punctuated::Pair;
use syn::{AngleBracketedGenericArguments, GenericArgument, Path, PathArguments, PathSegment, Type};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DamlType {
    ContractId,
    Int64,
    Decimal,
    Text,
    Timestamp,
    Party,
    Bool,
    Unit,
    Date,
    List(Box<DamlType>),
    TextMap(Box<DamlType>),
    Optional(Box<DamlType>),
    Data(String),
}

impl DamlType {
    pub fn from_type(ty: &Type) -> Self {
        match ty {
            Type::Path(s) => daml_type_from_path(&s.path),
            _ => panic!("only Path types allowed"),
        }
    }

    pub fn from_path(path: &Path) -> Self {
        daml_type_from_path(path)
    }

    pub fn name(&self) -> &str {
        match self {
            DamlType::ContractId => "DamlContractId",
            DamlType::Int64 => "DamlInt64",
            DamlType::Decimal => "DamlDecimal",
            DamlType::Text => "DamlText",
            DamlType::Timestamp => "DamlTimestamp",
            DamlType::Party => "DamlParty",
            DamlType::Bool => "DamlBool",
            DamlType::Unit => "DamlUnit",
            DamlType::Date => "DamlDate",
            DamlType::List(_) => "DamlList",
            DamlType::TextMap(_) => "DamlTextMap",
            DamlType::Optional(_) => "DamlOptional",
            DamlType::Data(data) => data,
        }
    }

    pub fn new_method(&self) -> (String, bool) {
        let with_param = if let DamlType::Unit = self {
            false
        } else {
            true
        };
        let new_method_name = format!("new_{}", self.get_type_method());
        (new_method_name, with_param)
    }

    pub fn try_method(&self) -> String {
        match self.get_type_method() {
            "decimal" => "try_decimal_clone".to_owned(),
            type_method => format!("try_{}", type_method),
        }
    }

    fn get_type_method(&self) -> &str {
        match self {
            DamlType::ContractId => "contract_id",
            DamlType::Int64 => "int64",
            DamlType::Decimal => "decimal",
            DamlType::Text => "text",
            DamlType::Timestamp => "timestamp",
            DamlType::Party => "party",
            DamlType::Bool => "bool",
            DamlType::Unit => "unit",
            DamlType::Date => "date",
            _ => panic!("internal error, get_type_method called for non-primitive type"),
        }
    }
}

/// Utility method to get a simple no-parameter type as a String from a Type.
pub fn data_type_string_from_type(ty: &Type) -> String {
    match ty {
        Type::Path(s) => data_type_string_from_path(&s.path),
        _ => panic!("only Path types allowed"),
    }
}

/// Utility method to get a simple no-parameter type as a String from a Path.
pub fn data_type_string_from_path(path: &Path) -> String {
    if path.segments.len() != 1 {
        panic!(format!(
            "expected exactly 1 segment, found {} in {:?}",
            &path.segments.len(),
            &path.segments.iter().map(|f| f.ident.to_string()).join("::")
        ))
    }
    match daml_type_from_path(path) {
        DamlType::Data(data) => data,
        _ => panic!("expected a single type"),
    }
}

fn daml_type_from_path(path: &Path) -> DamlType {
    let segments: Vec<_> = path.segments.iter().collect();
    match segments.as_slice() {
        [] => panic!("path has no segments"),
        [seg] => daml_type_from_path_segment(seg),
        _ => DamlType::Data(segments.iter().map(|f| f.ident.to_string()).join("::")),
    }
}

fn daml_type_from_path_segment(segment: &PathSegment) -> DamlType {
    if segment.arguments.is_empty() {
        daml_type_from_primitive_segment(&segment)
    } else {
        daml_type_from_parameterized_segment(&segment)
    }
}

fn daml_type_from_primitive_segment(segment: &PathSegment) -> DamlType {
    match segment.ident.to_string().as_ref() {
        "DamlContractId" => DamlType::ContractId,
        "DamlInt64" => DamlType::Int64,
        "DamlDecimal" => DamlType::Decimal,
        "DamlText" => DamlType::Text,
        "DamlTimestamp" => DamlType::Timestamp,
        "DamlParty" => DamlType::Party,
        "DamlBool" => DamlType::Bool,
        "DamlUnit" => DamlType::Unit,
        "DamlDate" => DamlType::Date,
        data_name => DamlType::Data(data_name.to_owned()),
    }
}

fn daml_type_from_parameterized_segment(segment: &PathSegment) -> DamlType {
    let ty = get_single_type_parameter(&segment.arguments);
    match segment.ident.to_string().as_ref() {
        "DamlList" => DamlType::List(Box::new(DamlType::from_type(ty))),
        "DamlTextMap" => DamlType::TextMap(Box::new(DamlType::from_type(ty))),
        "DamlOptional" => DamlType::Optional(Box::new(DamlType::from_type(ty))),
        _ => panic!("unexpected parameterized type, expected one of DamlList, DamlTextMap or DamlOptional"),
    }
}

fn get_single_type_parameter(path_args: &PathArguments) -> &Type {
    match path_args {
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            args,
            ..
        }) => match args.first() {
            Some(Pair::Punctuated(GenericArgument::Type(ty), _)) | Some(Pair::End(GenericArgument::Type(ty))) => ty,
            _ => panic!("failed to extract type from first AngleBracketed path argument"),
        },
        PathArguments::None => panic!("path argument is None"),
        PathArguments::Parenthesized(_) => panic!("path argument is Parenthesized"),
    }
}
