use crate::convert::interned::{InternableDottedName, InternableString, PackageInternedResolver};
use crate::convert::util::Required;
use crate::convert::wrapper::PayloadElementWrapper;
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use std::convert::{TryFrom, TryInto};

use crate::lf_protobuf::com::digitalasset::daml_lf_1::r#type::{Con, Sum, Var};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::{package_ref, PrimType};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::{ModuleRef, PackageRef, Type, TypeConName};

///
pub type DamlTypeWrapper<'a> = PayloadElementWrapper<'a, &'a DamlTypePayload<'a>>;

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

impl<'a> DamlTypePayload<'a> {
    pub fn name_for_error(&self) -> &'static str {
        match self {
            DamlTypePayload::ContractId(_) => "ContractId",
            DamlTypePayload::Int64 => "Int64",
            DamlTypePayload::Numeric => "Numeric",
            DamlTypePayload::Text => "Text",
            DamlTypePayload::Timestamp => "Timestamp",
            DamlTypePayload::Party => "Party",
            DamlTypePayload::Bool => "Bool",
            DamlTypePayload::Unit => "Unit",
            DamlTypePayload::Date => "Date",
            DamlTypePayload::List(_) => "List",
            DamlTypePayload::Update => "Update",
            DamlTypePayload::Scenario => "Scenario",
            DamlTypePayload::TextMap(_) => "TextMap",
            DamlTypePayload::Optional(_) => "Optional",
            DamlTypePayload::DataRef(_) => "DataRef",
            DamlTypePayload::Var(_) => "Var",
            DamlTypePayload::Arrow => "Arrow",
            DamlTypePayload::Any => "Any",
            DamlTypePayload::TypeRep => "TypeRep",
        }
    }
}

impl<'a> TryFrom<&'a Type> for DamlTypePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(ty: &'a Type) -> DamlLfConvertResult<Self> {
        match ty.sum.as_ref().req()? {
            Sum::Prim(prim) => match PrimType::from_i32(prim.prim).req()? {
                PrimType::Unit => Ok(DamlTypePayload::Unit),
                PrimType::Bool => Ok(DamlTypePayload::Bool),
                PrimType::Int64 => Ok(DamlTypePayload::Int64),
                PrimType::Numeric | PrimType::Decimal => Ok(DamlTypePayload::Numeric),
                PrimType::Text => Ok(DamlTypePayload::Text),
                PrimType::Timestamp => Ok(DamlTypePayload::Timestamp),
                PrimType::Party => Ok(DamlTypePayload::Party),
                PrimType::List =>
                    Ok(DamlTypePayload::List(Box::new(DamlTypePayload::try_from(prim.args.first().req()?)?))),
                PrimType::Update => Ok(DamlTypePayload::Update),
                PrimType::Scenario => Ok(DamlTypePayload::Scenario),
                PrimType::Date => Ok(DamlTypePayload::Date),
                PrimType::ContractId => match prim.args.as_slice() {
                    [Type {
                        sum: Some(Sum::Con(con)),
                    }] => Ok(DamlTypePayload::ContractId(Some(DamlDataRefPayload::try_from(con)?))),
                    args if !args.is_empty() => Err(DamlLfConvertError::UnexpectedContractIdTypeArguments),
                    _ => Ok(DamlTypePayload::ContractId(None)),
                },
                PrimType::Optional =>
                    Ok(DamlTypePayload::Optional(Box::new(DamlTypePayload::try_from(prim.args.first().req()?)?))),
                PrimType::Arrow => Ok(DamlTypePayload::Arrow),
                PrimType::Textmap =>
                    Ok(DamlTypePayload::TextMap(Box::new(DamlTypePayload::try_from(prim.args.first().req()?)?))),
                PrimType::Any => Ok(DamlTypePayload::Any),
                PrimType::TypeRep => Ok(DamlTypePayload::TypeRep),
            },
            Sum::Con(con) => Ok(DamlTypePayload::DataRef(DamlDataRefPayload::try_from(con)?)),
            Sum::Var(var) => Ok(DamlTypePayload::Var(DamlVarPayload::try_from(var)?)),
            Sum::Fun(_) => Err(DamlLfConvertError::UnsupportedType("Fun".to_owned())),
            Sum::Forall(_) => Err(DamlLfConvertError::UnsupportedType("Forall".to_owned())),
            Sum::Struct(_) => Err(DamlLfConvertError::UnsupportedType("Struct".to_owned())),
            Sum::Nat(_) => Err(DamlLfConvertError::UnsupportedType("Nat".to_owned())),
            Sum::Syn(_) => Err(DamlLfConvertError::UnsupportedType("Syn".to_owned())),
        }
    }
}

///
pub type DamlDataRefWrapper<'a> = PayloadElementWrapper<'a, &'a DamlDataRefPayload<'a>>;

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

impl<'a> TryFrom<&'a Con> for DamlDataRefPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(con: &'a Con) -> DamlLfConvertResult<Self> {
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
            } => Ok(Self::new(
                package_ref.try_into()?,
                InternableDottedName::from(module_name),
                InternableDottedName::from(data_name),
                args.iter().map(DamlTypePayload::try_from).collect::<DamlLfConvertResult<_>>()?,
            )),
            _ => Err(DamlLfConvertError::MissingRequiredField),
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
    pub fn resolve(&self, resolver: &'a impl PackageInternedResolver) -> DamlLfConvertResult<&'a str> {
        Ok(match self {
            DamlPackageRefPayload::This => resolver.package_id(),
            &DamlPackageRefPayload::PackageId(s) => s,
            &DamlPackageRefPayload::InternedId(i) => resolver.resolve_string(i)?,
        })
    }
}

impl<'a> TryFrom<&'a PackageRef> for DamlPackageRefPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(package_ref: &'a PackageRef) -> DamlLfConvertResult<Self> {
        Ok(match package_ref.sum.as_ref().req()? {
            package_ref::Sum::Self_(_) => DamlPackageRefPayload::This,
            package_ref::Sum::PackageIdStr(s) => DamlPackageRefPayload::PackageId(s.as_str()),
            &package_ref::Sum::PackageIdInternedStr(i) => DamlPackageRefPayload::InternedId(i),
        })
    }
}

///
pub type DamlVarWrapper<'a> = PayloadElementWrapper<'a, &'a DamlVarPayload<'a>>;

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

impl<'a> TryFrom<&'a Var> for DamlVarPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(var: &'a Var) -> DamlLfConvertResult<Self> {
        Ok(DamlVarPayload::new(
            InternableString::from(var.var.as_ref().req()?),
            var.args.iter().map(DamlTypePayload::try_from).collect::<DamlLfConvertResult<_>>()?,
        ))
    }
}
