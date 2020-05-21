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
    GenMap(Box<AttrType>, Box<AttrType>),
    Optional(Box<AttrType>),
    TyCon(String, Vec<String>, Vec<AttrType>),
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
        AttrType::TyCon(data, ..) => data,
        _ => panic!("expected a single type"),
    }
}

fn daml_type_from_path(path: &Path) -> AttrType {
    let segments: Vec<_> = path.segments.iter().collect();
    daml_type_from_segments(segments.as_slice())
}

fn daml_type_from_segments(segments: &[&PathSegment]) -> AttrType {
    let (last_segment, path) = split_segments(segments);
    let type_params = extract_type_parameters(&last_segment.arguments);
    match (last_segment.ident.to_string().as_ref(), type_params.as_slice()) {
        ("DamlInt64", _) => AttrType::Int64,
        ("DamlNumeric", _) => AttrType::Numeric,
        ("DamlText", _) => AttrType::Text,
        ("DamlTimestamp", _) => AttrType::Timestamp,
        ("DamlParty", _) => AttrType::Party,
        ("DamlBool", _) => AttrType::Bool,
        ("DamlUnit", _) => AttrType::Unit,
        ("DamlDate", _) => AttrType::Date,
        ("Box", &[ty]) => {
            let nested = AttrType::from_type(ty);
            match nested {
                AttrType::TyCon(..) => AttrType::Box(Box::new(nested)),
                _ => panic!("Box may only be applied to data types"),
            }
        },
        ("DamlList", &[ty]) => AttrType::List(Box::new(AttrType::from_type(ty))),
        ("DamlTextMap", &[ty]) => AttrType::TextMap(Box::new(AttrType::from_type(ty))),
        ("DamlGenMap", &[k, v]) => AttrType::GenMap(Box::new(AttrType::from_type(k)), Box::new(AttrType::from_type(v))),
        ("DamlOptional", &[ty]) => AttrType::Optional(Box::new(AttrType::from_type(ty))),
        ("DamlContractId", &[ty]) => AttrType::ContractId(Box::new(AttrType::from_type(ty))),
        ("DamlContractId", _) => AttrType::ContractId(Box::new(AttrType::Unit)),
        (data_name, type_arguments) => AttrType::TyCon(
            data_name.to_owned(),
            path,
            type_arguments.iter().map(|&arg| AttrType::from_type(arg)).collect(),
        ),
    }
}

fn split_segments<'a>(segments: &'a [&PathSegment]) -> (&'a PathSegment, Vec<String>) {
    match segments {
        [] => panic!("path has no segments"),
        [segment] => (segment, vec![]),
        [path @ .., last] => (last, path[1..].iter().map(|&s| s.ident.to_string()).collect()),
    }
}

fn extract_type_parameters(path_args: &PathArguments) -> Vec<&Type> {
    match path_args {
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            args,
            ..
        }) => args
            .iter()
            .filter_map(|arg| match arg {
                GenericArgument::Type(ty) => Some(ty),
                _ => None,
            })
            .collect::<Vec<_>>(),
        PathArguments::None => vec![],
        PathArguments::Parenthesized(_) => panic!("path argument is Parenthesized"),
    }
}
