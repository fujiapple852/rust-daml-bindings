use crate::convert::archive_payload::DamlArchiveWrapper;
use crate::convert::data_data_box_checker::DamlDataBoxChecker;
use crate::convert::data_payload::{DamlChoiceWrapper, DamlDataEnrichedPayload, DamlDataPayload, DamlDataWrapper};
use crate::convert::field_payload::{DamlFieldPayload, DamlFieldWrapper};
use crate::convert::interned::{InternableDottedName, PackageInternedResolver};
use crate::convert::module_payload::{DamlDefTypeSynWrapper, DamlFeatureFlagsPayload, DamlModuleWrapper};
use crate::convert::package_payload::{DamlPackagePayload, DamlPackageWrapper};
use crate::convert::resolver::resolve_tycon;
use crate::convert::type_payload::{
    DamlForallWrapper, DamlPackageRefPayload, DamlStructWrapper, DamlSynWrapper, DamlTyConNameWrapper,
    DamlTyConWrapper, DamlTypePayload, DamlTypeSynNameWrapper, DamlTypeWrapper, DamlVarWrapper,
};
use crate::convert::typevar_payload::{DamlKindPayload, DamlTypeVarWithKindPayload, DamlTypeVarWithKindWrapper};
use crate::convert::wrapper::{DamlPayloadParentContext, DamlPayloadParentContextType, PayloadElementWrapper};
#[cfg(feature = "full")]
use crate::element::DamlDefValue;
use crate::element::{
    DamlArchive, DamlArrow, DamlChoice, DamlData, DamlDefTypeSyn, DamlEnum, DamlFeatureFlags, DamlField, DamlForall,
    DamlKind, DamlLocalTyCon, DamlModule, DamlNonLocalTyCon, DamlPackage, DamlRecord, DamlStruct, DamlSyn,
    DamlTemplate, DamlTyCon, DamlTyConName, DamlType, DamlTypeSynName, DamlTypeVarWithKind, DamlVar, DamlVariant,
};
#[cfg(feature = "full")]
use crate::element::{DamlDefKey, DamlExpr};
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::LanguageFeatureVersion;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::TryFrom;

/// Convert from `DamlArchiveWrapper` to `DamlArchive`.
impl<'a> TryFrom<DamlArchiveWrapper<'a>> for DamlArchive<'a> {
    type Error = DamlLfConvertError;

    fn try_from(archive: DamlArchiveWrapper<'a>) -> DamlLfConvertResult<Self> {
        let packages: HashMap<_, _> = archive
            .archive
            .packages
            .values()
            .map(|package| Ok((Cow::from(package.package_id), DamlPackage::try_from(archive.with_package(package))?)))
            .collect::<DamlLfConvertResult<_>>()?;
        Ok(DamlArchive::new(Cow::from(archive.archive.name), packages))
    }
}

/// Convert from `DamlPackageWrapper` to `DamlPackage`.
impl<'a> TryFrom<DamlPackageWrapper<'a>> for DamlPackage<'a> {
    type Error = DamlLfConvertError;

    fn try_from(package: DamlPackageWrapper<'a>) -> DamlLfConvertResult<Self> {
        Ok(DamlPackage::new(
            Cow::from(&package.package.name),
            Cow::from(package.package.package_id),
            package.package.version.as_ref().map(Cow::from),
            package.package.language_version,
            from_modules(package.package.modules.values().map(|module| package.with_module(module)))?,
        ))
    }
}

/// Convert from `DamlModuleWrapper` to `DamlModule`.
impl<'a> TryFrom<&DamlModuleWrapper<'a>> for DamlModule<'a> {
    type Error = DamlLfConvertError;

    fn try_from(module: &DamlModuleWrapper<'a>) -> DamlLfConvertResult<Self> {
        let flags = DamlFeatureFlags::from(&module.module.flags);
        let path = module.module.path.resolve(module.package)?;
        let synonyms: Vec<_> = module
            .module
            .synonyms
            .iter()
            .map(|syn| DamlDefTypeSyn::try_from(module.wrap_def_type_syn(syn)))
            .collect::<DamlLfConvertResult<_>>()?;
        let data_types: Vec<DamlData<'_>> = module
            .module
            .data_types
            .iter()
            .map(|dt| DamlData::try_from(enriched_data(module.wrap_data(dt))?))
            .collect::<DamlLfConvertResult<_>>()?;
        #[cfg(feature = "full")]
        let values: Vec<_> = module
            .module
            .values
            .iter()
            .map(|val| DamlDefValue::try_from(&module.wrap_value(val)))
            .collect::<DamlLfConvertResult<_>>()?;
        Ok(DamlModule::new_leaf(
            path,
            flags,
            synonyms,
            data_types.into_iter().map(|dt| (dt.name_clone(), dt)).collect(),
            #[cfg(feature = "full")]
            values,
        ))
    }
}

/// Convert from `DamlFeatureFlagsPayload` to `DamlFeatureFlags`.
impl From<&DamlFeatureFlagsPayload> for DamlFeatureFlags {
    fn from(feature_flags: &DamlFeatureFlagsPayload) -> Self {
        Self::new(
            feature_flags.forbid_party_literals,
            feature_flags.dont_divulge_contract_ids_in_create_arguments,
            feature_flags.dont_disclose_non_consuming_choices_to_observers,
        )
    }
}

/// Convert from `DamlDefTypeSynWrapper` to `DamlDefTypeSyn`.
impl<'a> TryFrom<DamlDefTypeSynWrapper<'a>> for DamlDefTypeSyn<'a> {
    type Error = DamlLfConvertError;

    fn try_from(def_type_syn: DamlDefTypeSynWrapper<'a>) -> DamlLfConvertResult<Self> {
        let params = def_type_syn
            .payload
            .params
            .iter()
            .map(|param| DamlTypeVarWithKind::try_from(&def_type_syn.wrap(param)))
            .collect::<DamlLfConvertResult<_>>()?;
        let name = def_type_syn.payload.name.resolve(def_type_syn.context.package)?;
        let ty = DamlType::try_from(&def_type_syn.wrap(&def_type_syn.payload.ty))?;
        Ok(DamlDefTypeSyn::new(params, ty, name))
    }
}

/// Convert from `DamlDataWrapper` to `DamlData`.
impl<'a> TryFrom<DamlDataWrapper<'a>> for DamlData<'a> {
    type Error = DamlLfConvertError;

    fn try_from(data: DamlDataWrapper<'a>) -> DamlLfConvertResult<Self> {
        let resolver = data.context.package;
        Ok(match data.payload {
            DamlDataEnrichedPayload::Record(record) => {
                let name = record.name.resolve_last(resolver)?;
                let type_arguments = convert_type_var_arguments(data, &record.type_arguments)?;
                let fields = convert_fields(data, &record.fields)?;
                let serializable = record.serializable;
                DamlData::Record(DamlRecord::new(name, fields, type_arguments, serializable))
            },
            DamlDataEnrichedPayload::Template(template) => {
                let name = template.name.resolve_last(resolver)?;
                let module_path = data.context.module.path.resolve(resolver)?;
                let parent_data = match data.context.parent {
                    DamlPayloadParentContextType::Data(DamlDataPayload::Record(record)) => Ok(record),
                    _ => Err(DamlLfConvertError::UnexpectedData),
                }?;
                let fields = convert_fields(data, &parent_data.fields)?;
                let choices: Vec<_> = template
                    .choices
                    .iter()
                    .map(|choice| DamlChoice::try_from(data.wrap(choice)))
                    .collect::<DamlLfConvertResult<_>>()?;
                let param = template.param.resolve(data.context.package)?;
                #[cfg(feature = "full")]
                let precond = template.precond.as_ref().map(|pre| DamlExpr::try_from(&data.wrap(pre))).transpose()?;
                #[cfg(feature = "full")]
                let signatories = DamlExpr::try_from(&data.wrap(&template.signatories))?;
                #[cfg(feature = "full")]
                let agreement = DamlExpr::try_from(&data.wrap(&template.agreement))?;
                #[cfg(feature = "full")]
                let observers = DamlExpr::try_from(&data.wrap(&template.observers))?;
                #[cfg(feature = "full")]
                let key = template.key.as_ref().map(|k| DamlDefKey::try_from(&data.wrap(k))).transpose()?;
                let serializable = parent_data.serializable;
                DamlData::Template(Box::new(DamlTemplate::new(
                    name,
                    Cow::from(data.context.package.package_id),
                    module_path,
                    fields,
                    choices,
                    param,
                    #[cfg(feature = "full")]
                    precond,
                    #[cfg(feature = "full")]
                    signatories,
                    #[cfg(feature = "full")]
                    agreement,
                    #[cfg(feature = "full")]
                    observers,
                    #[cfg(feature = "full")]
                    key,
                    serializable,
                )))
            },
            DamlDataEnrichedPayload::Variant(variant) => {
                let name = variant.name.resolve_last(resolver)?;
                let type_arguments = convert_type_var_arguments(data, &variant.type_arguments)?;
                let fields = convert_fields(data, &variant.fields)?;
                let serializable = variant.serializable;
                DamlData::Variant(DamlVariant::new(name, fields, type_arguments, serializable))
            },
            DamlDataEnrichedPayload::Enum(data_enum) => {
                let name = data_enum.name.resolve_last(resolver)?;
                let type_arguments = convert_type_var_arguments(data, &data_enum.type_arguments)?;
                let constructors: Vec<Cow<'_, str>> = if data
                    .context
                    .package
                    .language_version
                    .supports_feature(&LanguageFeatureVersion::INTERNED_STRINGS)
                {
                    assert!(data_enum.constructors_str.is_empty(), "constructors_str should be empty!");
                    data.context.package.resolve_strings(data_enum.constructors_interned_str)?
                } else {
                    assert!(
                        data_enum.constructors_interned_str.is_empty(),
                        "constructors_interned_str should be empty!"
                    );
                    data_enum.constructors_str.iter().map(Cow::from).collect()
                };
                let serializable = data_enum.serializable;
                DamlData::Enum(DamlEnum::new(name, constructors, type_arguments, serializable))
            },
        })
    }
}

/// Convert from `DamlChoiceWrapper` to `DamlChoice`.
impl<'a> TryFrom<DamlChoiceWrapper<'a>> for DamlChoice<'a> {
    type Error = DamlLfConvertError;

    fn try_from(choice: DamlChoiceWrapper<'a>) -> DamlLfConvertResult<Self> {
        let name = choice.payload.name.resolve(choice.context.package)?;
        let target_data_wrapper = match &choice.payload.argument_type {
            DamlTypePayload::TyCon(tycon) => Ok(resolve_tycon(choice.wrap(tycon))?),
            _ => Err(DamlLfConvertError::UnexpectedType(
                "TyCon".to_owned(),
                choice.payload.argument_type.name_for_error().to_owned(),
            )),
        }?;
        let fields = match target_data_wrapper.payload {
            DamlDataEnrichedPayload::Record(record) => Ok(record
                .fields
                .iter()
                .map(|field| DamlField::try_from(target_data_wrapper.wrap(field)))
                .collect::<DamlLfConvertResult<_>>()?),
            _ => Err(DamlLfConvertError::UnexpectedChoiceData),
        }?;
        let return_type = DamlType::try_from(&choice.wrap(&choice.payload.return_type))?;
        Ok(DamlChoice::new(name, fields, return_type))
    }
}

/// Convert from `DamlFieldWrapper` to `DamlField`.
impl<'a> TryFrom<DamlFieldWrapper<'a>> for DamlField<'a> {
    type Error = DamlLfConvertError;

    fn try_from(field: DamlFieldWrapper<'a>) -> DamlLfConvertResult<Self> {
        Ok(DamlField::new(
            field.payload.name.resolve(field.context.package)?,
            DamlType::try_from(&field.wrap(&field.payload.ty))?,
        ))
    }
}

/// Convert from `DamlTypeWrapper` to `DamlType`.
impl<'a> TryFrom<&DamlTypeWrapper<'a>> for DamlType<'a> {
    type Error = DamlLfConvertError;

    fn try_from(daml_type: &DamlTypeWrapper<'a>) -> Result<Self, Self::Error> {
        Ok(match daml_type.payload {
            DamlTypePayload::ContractId(Some(ty)) =>
                DamlType::ContractId(Some(Box::new(DamlType::try_from(&daml_type.wrap(ty.as_ref()))?))),
            DamlTypePayload::ContractId(None) => DamlType::ContractId(None),
            DamlTypePayload::Int64 => DamlType::Int64,
            DamlTypePayload::Numeric(inner_type) =>
                DamlType::Numeric(Box::new(DamlType::try_from(&daml_type.wrap(inner_type.as_ref()))?)),
            DamlTypePayload::Text => DamlType::Text,
            DamlTypePayload::Timestamp => DamlType::Timestamp,
            DamlTypePayload::Party => DamlType::Party,
            DamlTypePayload::Bool => DamlType::Bool,
            DamlTypePayload::Unit => DamlType::Unit,
            DamlTypePayload::Date => DamlType::Date,
            DamlTypePayload::List(args) => DamlType::List(
                args.iter()
                    .map(|arg| DamlType::try_from(&daml_type.wrap(arg)))
                    .collect::<DamlLfConvertResult<Vec<_>>>()?,
            ),
            DamlTypePayload::Update => DamlType::Update,
            DamlTypePayload::Scenario => DamlType::Scenario,
            DamlTypePayload::TextMap(args) => DamlType::TextMap(
                args.iter()
                    .map(|arg| DamlType::try_from(&daml_type.wrap(arg)))
                    .collect::<DamlLfConvertResult<Vec<_>>>()?,
            ),
            DamlTypePayload::Optional(args) => DamlType::Optional(
                args.iter()
                    .map(|arg| DamlType::try_from(&daml_type.wrap(arg)))
                    .collect::<DamlLfConvertResult<Vec<_>>>()?,
            ),
            DamlTypePayload::TyCon(tycon_payload) => {
                let tycon_wrapper = daml_type.wrap(tycon_payload);
                let target_data_wrapper = resolve_tycon(tycon_wrapper)?;
                let tycon = DamlTyCon::try_from(&tycon_wrapper)?;
                match daml_type.context.parent {
                    DamlPayloadParentContextType::Data(_) => {
                        if DamlDataBoxChecker::should_box(enriched_data(daml_type.context)?, target_data_wrapper)? {
                            DamlType::BoxedTyCon(tycon)
                        } else {
                            DamlType::TyCon(tycon)
                        }
                    },
                    // We are not in a context with a DamlDataPayload and so we do not need to Box this reference
                    _ => DamlType::TyCon(tycon),
                }
            },
            DamlTypePayload::Var(var) => DamlType::Var(DamlVar::try_from(&daml_type.wrap(var))?),
            DamlTypePayload::Arrow => DamlType::Arrow,
            DamlTypePayload::Any => DamlType::Any,
            DamlTypePayload::TypeRep => DamlType::TypeRep,
            DamlTypePayload::Nat(n) => DamlType::Nat(*n),
            DamlTypePayload::Forall(forall) => DamlType::Forall(DamlForall::try_from(&daml_type.wrap(forall))?),
            DamlTypePayload::Struct(tuple) => DamlType::Struct(DamlStruct::try_from(&daml_type.wrap(tuple))?),
            DamlTypePayload::Syn(syn) => DamlType::Syn(DamlSyn::try_from(&daml_type.wrap(syn))?),
        })
    }
}

/// Convert from `DamlSynWrapper` to `DamlSyn`.
impl<'a> TryFrom<&DamlSynWrapper<'a>> for DamlSyn<'a> {
    type Error = DamlLfConvertError;

    fn try_from(syn: &DamlSynWrapper<'a>) -> Result<Self, Self::Error> {
        let tysyn = DamlTypeSynName::try_from(&syn.wrap(&syn.payload.tysyn))?;
        let args = syn
            .payload
            .args
            .iter()
            .map(|arg| DamlType::try_from(&syn.wrap(arg)))
            .collect::<DamlLfConvertResult<_>>()?;
        Ok(DamlSyn::new(tysyn, args))
    }
}

/// Convert from `DamlStructWrapper` to `DamlStruct`.
impl<'a> TryFrom<&DamlStructWrapper<'a>> for DamlStruct<'a> {
    type Error = DamlLfConvertError;

    fn try_from(tuple: &DamlStructWrapper<'a>) -> Result<Self, Self::Error> {
        let fields = tuple
            .payload
            .fields
            .iter()
            .map(|field| DamlField::try_from(tuple.wrap(field)))
            .collect::<DamlLfConvertResult<_>>()?;
        Ok(DamlStruct::new(fields))
    }
}

/// Convert from `ForallWrapper` to `DamlForall`.
impl<'a> TryFrom<&DamlForallWrapper<'a>> for DamlForall<'a> {
    type Error = DamlLfConvertError;

    fn try_from(forall: &DamlForallWrapper<'a>) -> Result<Self, Self::Error> {
        let vars = forall
            .payload
            .vars
            .iter()
            .map(|var| DamlTypeVarWithKind::try_from(&forall.wrap(var)))
            .collect::<DamlLfConvertResult<_>>()?;
        let body = DamlType::try_from(&forall.wrap(forall.payload.body.as_ref()))?;
        Ok(DamlForall::new(vars, Box::new(body)))
    }
}

/// Convert from `DamlTypeVarWrapper` to `DamlTypeVar`.
impl<'a> TryFrom<&DamlTypeVarWithKindWrapper<'a>> for DamlTypeVarWithKind<'a> {
    type Error = DamlLfConvertError;

    fn try_from(typevar: &DamlTypeVarWithKindWrapper<'a>) -> DamlLfConvertResult<Self> {
        Ok(DamlTypeVarWithKind::new(
            typevar.payload.var.resolve(typevar.context.package)?,
            DamlKind::from(&typevar.payload.kind),
        ))
    }
}

/// Convert from `DamlVarWrapper` to `DamlVar`.
impl<'a> TryFrom<&DamlVarWrapper<'a>> for DamlVar<'a> {
    type Error = DamlLfConvertError;

    fn try_from(var: &DamlVarWrapper<'a>) -> Result<Self, Self::Error> {
        let type_arguments = var
            .payload
            .type_arguments
            .iter()
            .map(|ty| DamlType::try_from(&var.wrap(ty)))
            .collect::<DamlLfConvertResult<_>>()?;
        Ok(DamlVar::new(var.payload.var.resolve(var.context.package)?, type_arguments))
    }
}

/// Convert from `DamlKindPayload` to `DamlKind`.
impl From<&DamlKindPayload> for DamlKind {
    fn from(kind: &DamlKindPayload) -> Self {
        match kind {
            DamlKindPayload::Star => DamlKind::Star,
            DamlKindPayload::Arrow(arrow) => DamlKind::Arrow(Box::new(DamlArrow::new(
                arrow.params.iter().map(DamlKind::from).collect(),
                DamlKind::from(&arrow.result),
            ))),
            DamlKindPayload::Nat => DamlKind::Nat,
        }
    }
}

/// DOCME
impl<'a> TryFrom<&DamlTyConNameWrapper<'a>> for DamlTyConName<'a> {
    type Error = DamlLfConvertError;

    fn try_from(tycon_name: &DamlTyConNameWrapper<'a>) -> DamlLfConvertResult<Self> {
        make_tycon_name(
            tycon_name.context,
            &tycon_name.payload.package_ref,
            tycon_name.payload.module_path,
            tycon_name.payload.data_name,
        )
    }
}

/// DOCME
impl<'a> TryFrom<&DamlTypeSynNameWrapper<'a>> for DamlTypeSynName<'a> {
    type Error = DamlLfConvertError;

    fn try_from(tysyn_name: &DamlTypeSynNameWrapper<'a>) -> DamlLfConvertResult<Self> {
        make_tycon_name(
            tysyn_name.context,
            &tysyn_name.payload.package_ref,
            tysyn_name.payload.module_path,
            tysyn_name.payload.data_name,
        )
    }
}

/// DOCME
impl<'a> TryFrom<&DamlTyConWrapper<'a>> for DamlTyCon<'a> {
    type Error = DamlLfConvertError;

    fn try_from(tycon: &DamlTyConWrapper<'a>) -> DamlLfConvertResult<Self> {
        let tycon_name = make_tycon_name(
            tycon.context,
            &tycon.payload.package_ref,
            tycon.payload.module_path,
            tycon.payload.data_name,
        )?;
        let type_arguments = convert_type_arguments(*tycon, &tycon.payload.type_arguments)?;
        Ok(DamlTyCon::new(tycon_name, type_arguments))
    }
}

/// Construct a `DamlModule` tree from a list of `DamlModuleWrapper`.
///
/// Each `DamlModulePayload` contains a `path` which indicates its position within the module namespace of a given
/// package.  Starting from a synthetic root, we use the module path information to construct a tree of modules to match
/// the shape of the namespace.
fn from_modules<'a, T: Iterator<Item = DamlModuleWrapper<'a>>>(modules: T) -> DamlLfConvertResult<DamlModule<'a>> {
    let mut root_module = DamlModule::new_root();
    for next_module in modules {
        let next_module: DamlModuleWrapper<'a> = next_module;
        let path = next_module.module.path.resolve_raw(next_module.package)?;
        let converted_module = DamlModule::try_from(&next_module)?;
        add_module_to_tree(&mut root_module, converted_module, &path);
    }
    Ok(root_module)
}

/// Add a module to the module tree.
///
/// Traverses the module tree from `node` by following the `remaining_path`, creating intermediate module nodes as we
/// go, and adds the `node_to_add` node as the leaf node.
fn add_module_to_tree<'a>(node: &mut DamlModule<'a>, node_to_add: DamlModule<'a>, remaining_path: &[&'a str]) {
    if let Some(&child_mod_name) = remaining_path.first() {
        add_module_to_tree(node.child_module_or_new(child_mod_name), node_to_add, &remaining_path[1..])
    } else {
        node.take_from(node_to_add);
    }
}

fn make_tycon_name<'a>(
    context: DamlPayloadParentContext<'a>,
    package_ref: &'a DamlPackageRefPayload<'a>,
    module_path: InternableDottedName<'a>,
    data_name: InternableDottedName<'a>,
) -> DamlLfConvertResult<DamlTyConName<'a>> {
    let source_resolver = context.package;
    let source_package_id = Cow::from(context.package.package_id);
    let source_package_name = Cow::from(context.package.name.as_str());
    let source_module_path = context.module.path.resolve(source_resolver)?;
    let target_package_id = package_ref.resolve(source_resolver)?;
    let target_package: &DamlPackagePayload<'_> = context
        .archive
        .package_by_id(&target_package_id)
        .ok_or_else(|| DamlLfConvertError::UnknownPackage(target_package_id.to_string()))?;
    let target_package_name = Cow::from(target_package.name.as_str());
    let target_module_path = module_path.resolve(source_resolver)?;
    let data_name = data_name.resolve_last(source_resolver)?;
    if target_package_name == source_package_name && target_module_path == source_module_path {
        Ok(DamlTyConName::Local(DamlLocalTyCon::new(
            data_name,
            target_package_id,
            target_package_name,
            target_module_path,
        )))
    } else {
        Ok(DamlTyConName::NonLocal(DamlNonLocalTyCon::new(
            data_name,
            source_package_id,
            source_package_name,
            source_module_path,
            target_package_id,
            target_package_name,
            target_module_path,
        )))
    }
}

fn enriched_data(context: DamlPayloadParentContext<'_>) -> DamlLfConvertResult<DamlDataWrapper<'_>> {
    Ok(DamlDataWrapper::with_data(context, DamlDataEnrichedPayload::from_data_wrapper(context)?))
}

fn convert_fields<'a, T: Copy>(
    wrapper: PayloadElementWrapper<'a, T>,
    fields: &'a [DamlFieldPayload<'a>],
) -> DamlLfConvertResult<Vec<DamlField<'a>>> {
    fields.iter().map(|field| DamlField::try_from(wrapper.wrap(field))).collect::<DamlLfConvertResult<_>>()
}

fn convert_type_arguments<'a, T: Copy>(
    wrapper: PayloadElementWrapper<'a, T>,
    type_arguments: &'a [DamlTypePayload<'a>],
) -> DamlLfConvertResult<Vec<DamlType<'a>>> {
    type_arguments.iter().map(|ty| DamlType::try_from(&wrapper.wrap(ty))).collect::<DamlLfConvertResult<_>>()
}

fn convert_type_var_arguments<'a, T: Copy>(
    wrapper: PayloadElementWrapper<'a, T>,
    type_var_arguments: &'a [DamlTypeVarWithKindPayload<'a>],
) -> DamlLfConvertResult<Vec<DamlTypeVarWithKind<'a>>> {
    type_var_arguments
        .iter()
        .map(|ty_arg| DamlTypeVarWithKind::try_from(&wrapper.wrap(ty_arg)))
        .collect::<DamlLfConvertResult<_>>()
}
