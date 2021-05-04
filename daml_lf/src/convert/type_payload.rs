use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

use crate::convert::field_payload::DamlFieldPayload;
use crate::convert::interned::{InternableDottedName, InternableString, PackageInternedResolver};
use crate::convert::typevar_payload::DamlTypeVarWithKindPayload;
use crate::convert::util::Required;
use crate::convert::wrapper::PayloadElementWrapper;
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::r#type::{Con, Forall, Struct, Sum, Syn, Var};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::{package_ref, PrimType, TypeSynName};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::{ModuleRef, PackageRef, Type, TypeConName};

///
pub type DamlTypeWrapper<'a> = PayloadElementWrapper<'a, &'a DamlTypePayload<'a>>;

#[derive(Debug)]
pub enum DamlTypePayload<'a> {
    ContractId(Option<Box<DamlTypePayload<'a>>>),
    Int64,
    Numeric(Box<DamlTypePayload<'a>>),
    Text,
    Timestamp,
    Party,
    Bool,
    Unit,
    Date,
    List(Vec<DamlTypePayload<'a>>),
    Update,
    Scenario,
    TextMap(Vec<DamlTypePayload<'a>>),
    GenMap(Vec<DamlTypePayload<'a>>),
    Optional(Vec<DamlTypePayload<'a>>),
    TyCon(DamlTyConPayload<'a>),
    Var(DamlVarPayload<'a>),
    Arrow,
    Any,
    TypeRep,
    AnyException,
    GeneralError,
    ArithmeticError,
    ContractError,
    Bignumeric,
    RoundingMode,
    Nat(u8),
    Forall(DamlForallPayload<'a>),
    Struct(DamlStructPayload<'a>),
    Syn(DamlSynPayload<'a>),
    Interned(i32),
}

impl<'a> DamlTypePayload<'a> {
    pub fn name_for_error(&self) -> &'static str {
        match self {
            DamlTypePayload::ContractId(_) => "ContractId",
            DamlTypePayload::Int64 => "Int64",
            DamlTypePayload::Numeric(_) => "Numeric",
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
            DamlTypePayload::GenMap(_) => "GenMap",
            DamlTypePayload::Optional(_) => "Optional",
            DamlTypePayload::TyCon(_) => "TyCon",
            DamlTypePayload::Var(_) => "Var",
            DamlTypePayload::Arrow => "Arrow",
            DamlTypePayload::Any => "Any",
            DamlTypePayload::TypeRep => "TypeRep",

            // TODO revisit when these types are stable in DAML LF 1.x
            DamlTypePayload::AnyException => "AnyException",
            DamlTypePayload::GeneralError => "GeneralError",
            DamlTypePayload::ArithmeticError => "ArithmeticError",
            DamlTypePayload::ContractError => "ContractError",
            DamlTypePayload::Bignumeric => "Bignumeric",
            DamlTypePayload::RoundingMode => "RoundingMode",
            DamlTypePayload::Nat(_) => "Nat",
            DamlTypePayload::Forall(_) => "Forall",
            DamlTypePayload::Struct(_) => "Struct",
            DamlTypePayload::Syn(_) => "Syn",
            DamlTypePayload::Interned(_) => "Interned",
        }
    }
}

/// The nat value to use for legacy Decimal types.
pub const LEGACY_DECIMAL_NAT: u8 = 10;

impl<'a> TryFrom<&'a Type> for DamlTypePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(ty: &'a Type) -> DamlLfConvertResult<Self> {
        match ty.sum.as_ref().req()? {
            Sum::Prim(prim) => match PrimType::from_i32(prim.prim).req()? {
                PrimType::Unit => Ok(DamlTypePayload::Unit),
                PrimType::Bool => Ok(DamlTypePayload::Bool),
                PrimType::Int64 => Ok(DamlTypePayload::Int64),
                PrimType::Numeric =>
                    Ok(DamlTypePayload::Numeric(Box::new(DamlTypePayload::try_from(prim.args.first().req()?)?))),
                PrimType::Decimal => Ok(DamlTypePayload::Numeric(Box::new(DamlTypePayload::Nat(LEGACY_DECIMAL_NAT)))),
                PrimType::Text => Ok(DamlTypePayload::Text),
                PrimType::Timestamp => Ok(DamlTypePayload::Timestamp),
                PrimType::Party => Ok(DamlTypePayload::Party),
                PrimType::List => Ok(DamlTypePayload::List(
                    prim.args.iter().map(DamlTypePayload::try_from).collect::<DamlLfConvertResult<Vec<_>>>()?,
                )),
                PrimType::Update => Ok(DamlTypePayload::Update),
                PrimType::Scenario => Ok(DamlTypePayload::Scenario),
                PrimType::Date => Ok(DamlTypePayload::Date),
                PrimType::ContractId => match prim.args.as_slice() {
                    [ty] => Ok(DamlTypePayload::ContractId(Some(Box::new(DamlTypePayload::try_from(ty)?)))),
                    args if !args.is_empty() => Err(DamlLfConvertError::UnexpectedContractIdTypeArguments),
                    _ => Ok(DamlTypePayload::ContractId(None)),
                },
                PrimType::Optional => Ok(DamlTypePayload::Optional(
                    prim.args.iter().map(DamlTypePayload::try_from).collect::<DamlLfConvertResult<Vec<_>>>()?,
                )),
                PrimType::Arrow => Ok(DamlTypePayload::Arrow),
                PrimType::Textmap => Ok(DamlTypePayload::TextMap(
                    prim.args.iter().map(DamlTypePayload::try_from).collect::<DamlLfConvertResult<Vec<_>>>()?,
                )),
                PrimType::Genmap => Ok(DamlTypePayload::GenMap(
                    prim.args.iter().map(DamlTypePayload::try_from).collect::<DamlLfConvertResult<Vec<_>>>()?,
                )),
                PrimType::Any => Ok(DamlTypePayload::Any),
                PrimType::TypeRep => Ok(DamlTypePayload::TypeRep),
                PrimType::AnyException => Ok(DamlTypePayload::AnyException),
                PrimType::GeneralError => Ok(DamlTypePayload::GeneralError),
                PrimType::ArithmeticError => Ok(DamlTypePayload::ArithmeticError),
                PrimType::ContractError => Ok(DamlTypePayload::ContractError),
                PrimType::Bignumeric => Ok(DamlTypePayload::Bignumeric),
                // TODO any generic args here?
                PrimType::RoundingMode => Ok(DamlTypePayload::RoundingMode),
            },
            Sum::Con(con) => Ok(DamlTypePayload::TyCon(DamlTyConPayload::try_from(con)?)),
            Sum::Var(var) => Ok(DamlTypePayload::Var(DamlVarPayload::try_from(var)?)),
            Sum::Nat(n) =>
                if *n >= 0 && *n <= 37 {
                    #[allow(clippy::cast_possible_truncation)]
                    Ok(DamlTypePayload::Nat(*n as u8))
                } else {
                    Err(DamlLfConvertError::NatOutOfRange(*n))
                },
            Sum::Forall(forall) => Ok(DamlTypePayload::Forall(DamlForallPayload::try_from(forall.as_ref())?)),
            Sum::Struct(tuple) => Ok(DamlTypePayload::Struct(DamlStructPayload::try_from(tuple)?)),
            Sum::Syn(syn) => Ok(DamlTypePayload::Syn(DamlSynPayload::try_from(syn)?)),
            Sum::Interned(i) => Ok(DamlTypePayload::Interned(*i)),
        }
    }
}

pub type DamlSynWrapper<'a> = PayloadElementWrapper<'a, &'a DamlSynPayload<'a>>;

#[derive(Debug)]
pub struct DamlSynPayload<'a> {
    pub tysyn: DamlTypeSynNamePayload<'a>,
    pub args: Vec<DamlTypePayload<'a>>,
}

impl<'a> DamlSynPayload<'a> {
    pub fn new(tysyn: DamlTypeSynNamePayload<'a>, args: Vec<DamlTypePayload<'a>>) -> Self {
        Self {
            tysyn,
            args,
        }
    }
}

impl<'a> TryFrom<&'a Syn> for DamlSynPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(syn: &'a Syn) -> DamlLfConvertResult<Self> {
        let tysyn = DamlTypeSynNamePayload::try_from(syn.tysyn.as_ref().req()?)?;
        let args = syn.args.iter().map(DamlTypePayload::try_from).collect::<DamlLfConvertResult<Vec<_>>>()?;
        Ok(DamlSynPayload::new(tysyn, args))
    }
}

pub type DamlTypeSynNameWrapper<'a> = PayloadElementWrapper<'a, &'a DamlTypeSynNamePayload<'a>>;

#[derive(Debug)]
pub struct DamlTypeSynNamePayload<'a> {
    pub package_ref: DamlPackageRefPayload<'a>,
    pub module_path: InternableDottedName<'a>,
    pub data_name: InternableDottedName<'a>,
}

impl<'a> DamlTypeSynNamePayload<'a> {
    pub fn new(
        package_ref: DamlPackageRefPayload<'a>,
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

impl<'a> TryFrom<&'a TypeSynName> for DamlTypeSynNamePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(tysyn: &'a TypeSynName) -> DamlLfConvertResult<Self> {
        match tysyn {
            TypeSynName {
                module:
                    Some(ModuleRef {
                        package_ref: Some(package_ref),
                        module_name: Some(module_name),
                    }),
                name: Some(data_name),
            } => Ok(Self::new(
                package_ref.try_into()?,
                InternableDottedName::from(module_name),
                InternableDottedName::from(data_name),
            )),
            _ => Err(DamlLfConvertError::MissingRequiredField),
        }
    }
}

///
pub type DamlStructWrapper<'a> = PayloadElementWrapper<'a, &'a DamlStructPayload<'a>>;

#[derive(Debug)]
pub struct DamlStructPayload<'a> {
    pub fields: Vec<DamlFieldPayload<'a>>,
}

impl<'a> DamlStructPayload<'a> {
    pub fn new(fields: Vec<DamlFieldPayload<'a>>) -> Self {
        Self {
            fields,
        }
    }
}

impl<'a> TryFrom<&'a Struct> for DamlStructPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(tuple: &'a Struct) -> DamlLfConvertResult<Self> {
        let fields = tuple.fields.iter().map(DamlFieldPayload::try_from).collect::<DamlLfConvertResult<Vec<_>>>()?;
        Ok(DamlStructPayload::new(fields))
    }
}

///
pub type DamlForallWrapper<'a> = PayloadElementWrapper<'a, &'a DamlForallPayload<'a>>;

#[derive(Debug)]
pub struct DamlForallPayload<'a> {
    pub vars: Vec<DamlTypeVarWithKindPayload<'a>>,
    pub body: Box<DamlTypePayload<'a>>,
}

impl<'a> DamlForallPayload<'a> {
    pub fn new(vars: Vec<DamlTypeVarWithKindPayload<'a>>, body: Box<DamlTypePayload<'a>>) -> Self {
        Self {
            vars,
            body,
        }
    }
}

impl<'a> TryFrom<&'a Forall> for DamlForallPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(forall: &'a Forall) -> DamlLfConvertResult<Self> {
        let vars =
            forall.vars.iter().map(DamlTypeVarWithKindPayload::try_from).collect::<DamlLfConvertResult<Vec<_>>>()?;
        let body = DamlTypePayload::try_from(forall.body.as_ref().req()?.as_ref())?;
        Ok(DamlForallPayload::new(vars, Box::new(body)))
    }
}

///
pub type DamlTyConWrapper<'a> = PayloadElementWrapper<'a, &'a DamlTyConPayload<'a>>;

#[derive(Debug)]
pub struct DamlTyConPayload<'a> {
    pub package_ref: DamlPackageRefPayload<'a>,
    pub module_path: InternableDottedName<'a>,
    pub data_name: InternableDottedName<'a>,
    pub type_arguments: Vec<DamlTypePayload<'a>>,
}

impl<'a> DamlTyConPayload<'a> {
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

impl<'a> TryFrom<&'a Con> for DamlTyConPayload<'a> {
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

pub type DamlTyConNameWrapper<'a> = PayloadElementWrapper<'a, &'a DamlTyConNamePayload<'a>>;

#[derive(Debug)]
pub struct DamlTyConNamePayload<'a> {
    pub package_ref: DamlPackageRefPayload<'a>,
    pub module_path: InternableDottedName<'a>,
    pub data_name: InternableDottedName<'a>,
}

impl<'a> DamlTyConNamePayload<'a> {
    pub fn new(
        package_ref: DamlPackageRefPayload<'a>,
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

impl<'a> TryFrom<&'a TypeConName> for DamlTyConNamePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(con: &'a TypeConName) -> DamlLfConvertResult<Self> {
        match con {
            TypeConName {
                module:
                    Some(ModuleRef {
                        package_ref: Some(package_ref),
                        module_name: Some(module_name),
                    }),
                name: Some(data_name),
            } => Ok(Self::new(
                package_ref.try_into()?,
                InternableDottedName::from(module_name),
                InternableDottedName::from(data_name),
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
    pub fn resolve(&self, resolver: &'a impl PackageInternedResolver) -> DamlLfConvertResult<Cow<'a, str>> {
        Ok(match self {
            DamlPackageRefPayload::This => Cow::from(resolver.package_id()),
            &DamlPackageRefPayload::PackageId(s) => Cow::from(s),
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
