use crate::convert::archive::wrapper::payload::*;
use daml_lf::protobuf_autogen::daml_lf_1::r#type::{Con, Sum, Var};
use daml_lf::protobuf_autogen::daml_lf_1::{package_ref, PrimType};
use daml_lf::protobuf_autogen::daml_lf_1::{ModuleRef, PackageRef, Type, TypeConName};

#[derive(Debug)]
pub enum DamlTypePayload<'a> {
    ContractId(Option<DamlDataRefPayload<'a>>),
    Int64,
    Numeric,
    Text,
    Timestamp,
    Party,
    Bool,
    Unit,
    Date,
    List(Box<DamlTypePayload<'a>>),
    Update,
    Scenario,
    TextMap(Box<DamlTypePayload<'a>>),
    Optional(Box<DamlTypePayload<'a>>),
    DataRef(DamlDataRefPayload<'a>),
    Var(DamlVarPayload<'a>),
    Arrow,
    Any,
    TypeRep,
}

impl<'a> From<&'a Type> for DamlTypePayload<'a> {
    fn from(ty: &'a Type) -> Self {
        match ty.sum.as_ref().expect("Type.sum") {
            Sum::Prim(prim) => match PrimType::from_i32(prim.prim).expect("Prim.prim") {
                PrimType::Unit => DamlTypePayload::Unit,
                PrimType::Bool => DamlTypePayload::Bool,
                PrimType::Int64 => DamlTypePayload::Int64,
                PrimType::Numeric | PrimType::Decimal => DamlTypePayload::Numeric,
                PrimType::Text => DamlTypePayload::Text,
                PrimType::Timestamp => DamlTypePayload::Timestamp,
                PrimType::Party => DamlTypePayload::Party,
                PrimType::List =>
                    DamlTypePayload::List(Box::new(DamlTypePayload::from(prim.args.first().expect("Prim.args")))),
                PrimType::Update => DamlTypePayload::Update,
                PrimType::Scenario => DamlTypePayload::Scenario,
                PrimType::Date => DamlTypePayload::Date,
                PrimType::ContractId => match prim.args.as_slice() {
                    [Type {
                        sum: Some(Sum::Con(con)),
                    }] => DamlTypePayload::ContractId(Some(DamlDataRefPayload::from(con))),
                    args if !args.is_empty() => panic!("ContractId with multiple type constructor arguments"),
                    _ => DamlTypePayload::ContractId(None),
                },
                PrimType::Optional =>
                    DamlTypePayload::Optional(Box::new(DamlTypePayload::from(prim.args.first().expect("Prim.args")))),
                PrimType::Arrow => DamlTypePayload::Arrow,
                PrimType::Map =>
                    DamlTypePayload::TextMap(Box::new(DamlTypePayload::from(prim.args.first().expect("Prim.args")))),
                PrimType::Any => DamlTypePayload::Any,
                PrimType::TypeRep => DamlTypePayload::TypeRep,
            },
            Sum::Con(con) => DamlTypePayload::DataRef(DamlDataRefPayload::from(con)),
            Sum::Var(var) => DamlTypePayload::Var(DamlVarPayload::from(var)),
            _ => panic!("unsupported type"),
        }
    }
}

#[derive(Debug)]
pub struct DamlDataRefPayload<'a> {
    pub package_ref: DamlPackageRefPayload<'a>,
    pub module_path: InternableDottedName<'a>,
    pub data_name: InternableDottedName<'a>,
    pub type_arguments: Vec<DamlTypePayload<'a>>,
}

impl<'a> DamlDataRefPayload<'a> {
    pub fn new(
        package_ref: DamlPackageRefPayload<'a>,
        module_path: InternableDottedName<'a>,
        data_name: InternableDottedName<'a>,
        type_arguments: Vec<DamlTypePayload<'a>>,
    ) -> Self {
        Self {
            package_ref,
            module_path,
            data_name,
            type_arguments,
        }
    }
}

impl<'a> From<&'a Con> for DamlDataRefPayload<'a> {
    fn from(con: &'a Con) -> Self {
        match con {
            Con {
                tycon:
                    Some(TypeConName {
                        module:
                            Some(ModuleRef {
                                package_ref: Some(package_ref),
                                module_name: Some(module_name),
                            }),
                        name: Some(data_name),
                    }),
                args,
            } => Self::new(
                package_ref.into(),
                InternableDottedName::from(module_name),
                InternableDottedName::from(data_name),
                args.iter().map(DamlTypePayload::from).collect(),
            ),
            _ => panic!("Con.tycon"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DamlPackageRefPayload<'a> {
    This,
    PackageId(&'a str),
    InternedId(i32),
}

impl<'a> DamlPackageRefPayload<'a> {
    pub fn resolve(&self, resolver: &'a impl PackageInternedResolver) -> Option<&'a str> {
        match self {
            DamlPackageRefPayload::This => Some(resolver.package_id()),
            &DamlPackageRefPayload::PackageId(s) => Some(s),
            &DamlPackageRefPayload::InternedId(i) => resolver.interned_strings().get(i as usize).map(AsRef::as_ref),
        }
    }
}

impl<'a> From<&'a PackageRef> for DamlPackageRefPayload<'a> {
    fn from(package_ref: &'a PackageRef) -> Self {
        match package_ref.sum.as_ref().expect("PackageRef.sum") {
            package_ref::Sum::Self_(_) => DamlPackageRefPayload::This,
            package_ref::Sum::PackageIdStr(s) => DamlPackageRefPayload::PackageId(s.as_str()),
            &package_ref::Sum::PackageIdInternedStr(i) => DamlPackageRefPayload::InternedId(i),
        }
    }
}

#[derive(Debug)]
pub struct DamlVarPayload<'a> {
    pub var: InternableString<'a>,
    pub type_arguments: Vec<DamlTypePayload<'a>>,
}

impl<'a> DamlVarPayload<'a> {
    pub fn new(var: InternableString<'a>, type_arguments: Vec<DamlTypePayload<'a>>) -> Self {
        Self {
            var,
            type_arguments,
        }
    }
}

impl<'a> From<&'a Var> for DamlVarPayload<'a> {
    fn from(var: &'a Var) -> Self {
        DamlVarPayload::new(
            InternableString::from(var.var.as_ref().expect("Var.var")),
            var.args.iter().map(DamlTypePayload::from).collect(),
        )
    }
}
