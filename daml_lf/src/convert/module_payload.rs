use crate::convert::archive_payload::DamlArchivePayload;
use crate::convert::data_payload::{DamlDataPayload, DamlTemplatePayload};
#[cfg(feature = "full")]
use crate::convert::defvalue_payload::{DamlDefValuePayload, DamlDefValueWrapper};
use crate::convert::interned::InternableDottedName;
use crate::convert::package_payload::DamlPackagePayload;
use crate::convert::type_payload::DamlTypePayload;
use crate::convert::typevar_payload::DamlTypeVarWithKindPayload;
use crate::convert::util::Required;
use crate::convert::wrapper::{DamlPayloadParentContext, DamlPayloadParentContextType, PayloadElementWrapper};
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::{DefTypeSyn, FeatureFlags, Module};
use std::collections::HashMap;
use std::convert::TryFrom;

///
#[derive(Debug, Clone, Copy)]
pub struct DamlModuleWrapper<'a> {
    pub archive: &'a DamlArchivePayload<'a>,
    pub package: &'a DamlPackagePayload<'a>,
    pub module: &'a DamlModulePayload<'a>,
}

impl<'a> DamlModuleWrapper<'a> {
    /// DOCME
    pub const fn wrap_data(self, data: &'a DamlDataPayload<'_>) -> DamlPayloadParentContext<'a> {
        DamlPayloadParentContext {
            archive: self.archive,
            package: self.package,
            module: self.module,
            parent: DamlPayloadParentContextType::Data(data),
        }
    }

    /// DOCME
    pub const fn wrap_def_type_syn(self, def_type_syn: &'a DamlDefTypeSynPayload<'_>) -> DamlDefTypeSynWrapper<'a> {
        DamlDefTypeSynWrapper {
            context: DamlPayloadParentContext {
                archive: self.archive,
                package: self.package,
                module: self.module,
                parent: DamlPayloadParentContextType::DefTypeSyn(def_type_syn),
            },
            payload: def_type_syn,
        }
    }

    #[cfg(feature = "full")]
    pub fn wrap_value(self, def_value: &'a DamlDefValuePayload<'_>) -> DamlDefValueWrapper<'a> {
        DamlDefValueWrapper {
            context: DamlPayloadParentContext {
                archive: self.archive,
                package: self.package,
                module: self.module,
                parent: DamlPayloadParentContextType::Value(def_value),
            },
            payload: def_value,
        }
    }
}

#[derive(Debug)]
pub struct DamlModulePayload<'a> {
    pub flags: DamlFeatureFlagsPayload,
    pub synonyms: Vec<DamlDefTypeSynPayload<'a>>,
    pub data_types: Vec<DamlDataPayload<'a>>,
    pub templates: HashMap<InternableDottedName<'a>, DamlTemplatePayload<'a>>,
    #[cfg(feature = "full")]
    pub values: Vec<DamlDefValuePayload<'a>>,
    pub path: InternableDottedName<'a>,
}

impl<'a> PartialEq for DamlModulePayload<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<'a> TryFrom<&'a Module> for DamlModulePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(module: &'a Module) -> DamlLfConvertResult<Self> {
        let flags = DamlFeatureFlagsPayload::from(module.flags.as_ref().req()?);
        let synonyms =
            module.synonyms.iter().map(DamlDefTypeSynPayload::try_from).collect::<DamlLfConvertResult<_>>()?;
        let path = InternableDottedName::from(module.name.as_ref().req()?);
        let templates = module
            .templates
            .iter()
            .map(DamlTemplatePayload::try_from)
            .map(|tr| tr.map(|t| (t.name, t)))
            .collect::<DamlLfConvertResult<_>>()?;
        let data_types =
            module.data_types.iter().map(DamlDataPayload::try_from).collect::<DamlLfConvertResult<Vec<_>>>()?;
        #[cfg(feature = "full")]
        let values =
            module.values.iter().map(DamlDefValuePayload::try_from).collect::<DamlLfConvertResult<Vec<_>>>()?;
        Ok(Self {
            flags,
            synonyms,
            data_types,
            templates,
            #[cfg(feature = "full")]
            values,
            path,
        })
    }
}

impl<'a> DamlModulePayload<'a> {
    /// Get a named template from this module.
    pub fn template(&self, name: &InternableDottedName<'a>) -> Option<&DamlTemplatePayload<'_>> {
        self.templates.get(name)
    }
}

pub type DamlDefTypeSynWrapper<'a> = PayloadElementWrapper<'a, &'a DamlDefTypeSynPayload<'a>>;

#[derive(Debug)]
pub struct DamlDefTypeSynPayload<'a> {
    pub params: Vec<DamlTypeVarWithKindPayload<'a>>,
    pub ty: DamlTypePayload<'a>,
    pub name: InternableDottedName<'a>,
}

impl<'a> DamlDefTypeSynPayload<'a> {
    pub fn new(
        params: Vec<DamlTypeVarWithKindPayload<'a>>,
        ty: DamlTypePayload<'a>,
        name: InternableDottedName<'a>,
    ) -> Self {
        Self {
            params,
            ty,
            name,
        }
    }
}

impl<'a> TryFrom<&'a DefTypeSyn> for DamlDefTypeSynPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(def_type_syn: &'a DefTypeSyn) -> DamlLfConvertResult<Self> {
        let params =
            def_type_syn.params.iter().map(DamlTypeVarWithKindPayload::try_from).collect::<DamlLfConvertResult<_>>()?;
        let name = InternableDottedName::from(def_type_syn.name.as_ref().req()?);
        let ty = DamlTypePayload::try_from(def_type_syn.r#type.as_ref().req()?)?;
        Ok(Self::new(params, ty, name))
    }
}

#[derive(Debug)]
pub struct DamlFeatureFlagsPayload {
    pub forbid_party_literals: bool,
    pub dont_divulge_contract_ids_in_create_arguments: bool,
    pub dont_disclose_non_consuming_choices_to_observers: bool,
}

impl DamlFeatureFlagsPayload {
    pub fn new(
        forbid_party_literals: bool,
        dont_divulge_contract_ids_in_create_arguments: bool,
        dont_disclose_non_consuming_choices_to_observers: bool,
    ) -> Self {
        Self {
            forbid_party_literals,
            dont_divulge_contract_ids_in_create_arguments,
            dont_disclose_non_consuming_choices_to_observers,
        }
    }
}

impl From<&FeatureFlags> for DamlFeatureFlagsPayload {
    fn from(feature_flags: &FeatureFlags) -> Self {
        Self {
            forbid_party_literals: feature_flags.forbid_party_literals,
            dont_divulge_contract_ids_in_create_arguments: feature_flags.dont_divulge_contract_ids_in_create_arguments,
            dont_disclose_non_consuming_choices_to_observers: feature_flags
                .dont_disclose_non_consuming_choices_to_observers,
        }
    }
}
