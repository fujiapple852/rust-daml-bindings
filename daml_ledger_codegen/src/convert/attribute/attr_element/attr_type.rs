use itertools::Itertools;
use syn::{AngleBracketedGenericArguments, GenericArgument, Path, PathArguments, PathSegment, Type};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum AttrType {
    ContractId(Box<AttrType>),
    Int64,
    Numeric,
    Text,
    Timestamp,
    Party,
    Bool,
    Unit,
    Date,
    Box(Box<AttrType>),
    List(Box<AttrType>),
    TextMap(Box<AttrType>),
    Optional(Box<AttrType>),
    Data(String, Vec<String>),
}

impl AttrType {
    pub fn from_type(ty: &Type) -> Self {
        match ty {
            Type::Path(s) => daml_type_from_path(&s.path),
            _ => panic!("only Path types allowed"),
        }
    }

    pub fn from_path(path: &Path) -> Self {
        daml_type_from_path(path)
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
        // TODO ignoring path here
        AttrType::Data(data, _) => data,
        _ => panic!("expected a single type"),
    }
}

fn daml_type_from_path(path: &Path) -> AttrType {
    let segments: Vec<_> = path.segments.iter().collect();
    match segments.as_slice() {
        [] => panic!("path has no segments"),
        [seg] => daml_type_from_path_segment(seg),
        _ =>
            if segments.first().unwrap().ident == "crate" {
                let data_name = segments.last().unwrap().ident.to_string();
                let path: Vec<String> = segments[1..segments.len() - 1].iter().map(|s| s.ident.to_string()).collect();
                AttrType::Data(data_name, path)
            } else {
                panic!("data type path must be absolute and begin with crate::")
            },
    }
}

fn daml_type_from_path_segment(segment: &PathSegment) -> AttrType {
    if segment.arguments.is_empty() {
        daml_type_from_primitive_segment(&segment)
    } else {
        daml_type_from_parameterized_segment(&segment)
    }
}

fn daml_type_from_primitive_segment(segment: &PathSegment) -> AttrType {
    match segment.ident.to_string().as_ref() {
        "DamlContractId" => AttrType::ContractId(Box::new(AttrType::Unit)),
        "DamlInt64" => AttrType::Int64,
        "DamlNumeric" => AttrType::Numeric,
        "DamlText" => AttrType::Text,
        "DamlTimestamp" => AttrType::Timestamp,
        "DamlParty" => AttrType::Party,
        "DamlBool" => AttrType::Bool,
        "DamlUnit" => AttrType::Unit,
        "DamlDate" => AttrType::Date,
        data_name => AttrType::Data(data_name.to_owned(), vec![]),
    }
}

fn daml_type_from_parameterized_segment(segment: &PathSegment) -> AttrType {
    let ty = get_single_type_parameter(&segment.arguments);
    match segment.ident.to_string().as_ref() {
        "Box" => {
            let nested = AttrType::from_type(ty);
            match nested {
                AttrType::Data(..) => AttrType::Box(Box::new(nested)),
                _ => panic!("Box may only be applied to data types"),
            }
        },
        "DamlList" => AttrType::List(Box::new(AttrType::from_type(ty))),
        "DamlTextMap" => AttrType::TextMap(Box::new(AttrType::from_type(ty))),
        "DamlOptional" => AttrType::Optional(Box::new(AttrType::from_type(ty))),
        "DamlContractId" => AttrType::ContractId(Box::new(AttrType::from_type(ty))),
        _ => panic!(format!(
            "unexpected parameterized type {}, expected one of Box, DamlList, DamlTextMap, DamlOptional or \
             DamlContractId",
            segment.ident.to_string()
        )),
    }
}

fn get_single_type_parameter(path_args: &PathArguments) -> &Type {
    match path_args {
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            args,
            ..
        }) => match args.first() {
            Some(GenericArgument::Type(ty)) => ty,
            _ => panic!("failed to extract type from first AngleBracketed path argument"),
        },
        PathArguments::None => panic!("path argument is None"),
        PathArguments::Parenthesized(_) => panic!("path argument is Parenthesized"),
    }
}
