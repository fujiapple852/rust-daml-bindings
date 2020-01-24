use crate::convert::archive::wrapper::payload::*;
use daml_lf::protobuf_autogen::daml_lf_1::r#type::*;
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
    Var,
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
                PrimType::ContractId => {
                    if let Some(Sum::Con(Con {
                        tycon: Some(tcn),
                        ..
                    })) = prim.args.first().unwrap().sum.as_ref()
                    {
                        DamlTypePayload::ContractId(Some(DamlDataRefPayload::from(tcn)))
                    } else {
                        DamlTypePayload::ContractId(None)
                    }
                },
                PrimType::Optional =>
                    DamlTypePayload::Optional(Box::new(DamlTypePayload::from(prim.args.first().expect("Prim.args")))),
                PrimType::Arrow => DamlTypePayload::Arrow,
                PrimType::Map =>
                    DamlTypePayload::TextMap(Box::new(DamlTypePayload::from(prim.args.first().expect("Prim.args")))),
                PrimType::Any => DamlTypePayload::Any,
                PrimType::TypeRep => DamlTypePayload::TypeRep,
            },
            Sum::Con(Con {
                tycon: Some(tcn),
                ..
            }) => DamlTypePayload::DataRef(DamlDataRefPayload::from(tcn)),
            Sum::Var(_) => DamlTypePayload::Var,
            _ => panic!("unsupported type"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DamlDataRefPayload<'a> {
    pub package_ref: DamlPackageRef<'a>,
    pub module_path: InternableDottedName<'a>,
    pub data_name: InternableDottedName<'a>,
}

impl<'a> DamlDataRefPayload<'a> {
    pub fn new(
        package_ref: DamlPackageRef<'a>,
        module_path: InternableDottedName<'a>,
        data_name: InternableDottedName<'a>,
    ) -> Self {
        Self {
            package_ref,
            module_path,
            data_name,
        }
    }
}

impl<'a> From<&'a TypeConName> for DamlDataRefPayload<'a> {
    fn from(type_con_name: &'a TypeConName) -> Self {
        match type_con_name {
            TypeConName {
                module:
                    Some(ModuleRef {
                        package_ref: Some(package_ref),
                        module_name: Some(module_name),
                    }),
                name: Some(data_name),
            } => Self::new(
                package_ref.into(),
                InternableDottedName::from(module_name),
                InternableDottedName::from(data_name),
            ),
            _ => panic!("TypeConName"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DamlPackageRef<'a> {
    This,
    PackageId(&'a str),
    InternedId(i32),
}

impl<'a> DamlPackageRef<'a> {
    pub fn resolve(&self, resolver: &'a impl PackageInternedResolver) -> Option<&'a str> {
        match self {
            DamlPackageRef::This => Some(resolver.package_id()),
            &DamlPackageRef::PackageId(s) => Some(s),
            &DamlPackageRef::InternedId(i) => resolver.interned_strings().get(i as usize).map(AsRef::as_ref),
        }
    }
}

impl<'a> From<&'a PackageRef> for DamlPackageRef<'a> {
    fn from(package_ref: &'a PackageRef) -> Self {
        match package_ref.sum.as_ref().expect("PackageRef.sum") {
            package_ref::Sum::Self_(_) => DamlPackageRef::This,
            package_ref::Sum::PackageIdStr(s) => DamlPackageRef::PackageId(s.as_str()),
            &package_ref::Sum::PackageIdInternedStr(i) => DamlPackageRef::InternedId(i),
        }
    }
}
