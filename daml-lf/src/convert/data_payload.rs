#[cfg(feature = "full")]
use crate::convert::expr_payload::{DamlExprPayload, DamlKeyExprPayload};
use crate::convert::field_payload::DamlFieldPayload;
use crate::convert::interned::{InternableDottedName, InternableString};
use crate::convert::type_payload::DamlTypePayload;
use crate::convert::typevar_payload::DamlTypeVarWithKindPayload;
use crate::convert::util::Required;
use crate::convert::wrapper::{DamlPayloadParentContext, DamlPayloadParentContextType, PayloadElementWrapper};
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::lf_protobuf::com::daml::daml_lf_1::def_data_type::DataCons;
use crate::lf_protobuf::com::daml::daml_lf_1::def_template::DefKey;
use crate::lf_protobuf::com::daml::daml_lf_1::{DefDataType, DefTemplate, TemplateChoice};
use std::convert::TryFrom;

///
pub type DamlDataWrapper<'a> = PayloadElementWrapper<'a, DamlDataEnrichedPayload<'a>>;

#[derive(Debug)]
pub enum DamlDataPayload<'a> {
    Record(DamlRecordPayload<'a>),
    Variant(DamlVariantPayload<'a>),
    Enum(DamlEnumPayload<'a>),
}

impl<'a> PartialEq for DamlDataPayload<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl<'a> DamlDataPayload<'a> {
    pub fn new_record(daml_record: impl Into<DamlRecordPayload<'a>>) -> Self {
        DamlDataPayload::Record(daml_record.into())
    }

    pub fn new_variant(daml_variant: impl Into<DamlVariantPayload<'a>>) -> Self {
        DamlDataPayload::Variant(daml_variant.into())
    }

    pub fn new_enum(daml_enum: impl Into<DamlEnumPayload<'a>>) -> Self {
        DamlDataPayload::Enum(daml_enum.into())
    }

    pub fn name(&self) -> &InternableDottedName<'_> {
        match self {
            DamlDataPayload::Record(record) => &record.name,
            DamlDataPayload::Variant(variant) => &variant.name,
            DamlDataPayload::Enum(daml_enum) => &daml_enum.name,
        }
    }
}

impl<'a> TryFrom<&'a DefDataType> for DamlDataPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(def_data_type: &'a DefDataType) -> DamlLfConvertResult<Self> {
        let name = InternableDottedName::from(def_data_type.name.as_ref().req()?);
        let type_params: Vec<_> = def_data_type
            .params
            .iter()
            .map(DamlTypeVarWithKindPayload::try_from)
            .collect::<DamlLfConvertResult<_>>()?;
        let serializable = def_data_type.serializable;
        Ok(match def_data_type.data_cons.as_ref().req()? {
            DataCons::Record(fields) => Self::new_record(DamlRecordPayload::new(
                name,
                fields.fields.iter().map(DamlFieldPayload::try_from).collect::<DamlLfConvertResult<_>>()?,
                type_params,
                serializable,
            )),
            DataCons::Variant(fields) => Self::new_variant(DamlVariantPayload::new(
                name,
                fields.fields.iter().map(DamlFieldPayload::try_from).collect::<DamlLfConvertResult<_>>()?,
                type_params,
                serializable,
            )),
            DataCons::Enum(constructors) => Self::new_enum(DamlEnumPayload::new(
                name,
                constructors.constructors_str.as_slice(),
                constructors.constructors_interned_str.as_slice(),
                type_params,
                serializable,
            )),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamlDataEnrichedPayload<'a> {
    Record(&'a DamlRecordPayload<'a>),
    Template(&'a DamlTemplatePayload<'a>),
    Variant(&'a DamlVariantPayload<'a>),
    Enum(&'a DamlEnumPayload<'a>),
}

impl<'a> DamlDataEnrichedPayload<'a> {
    /// Create a `DamlDataEnrichedPayload` from a `DamlPayloadDataWrapper`.
    ///
    /// A `DamlPayloadDataWrapper` contains a `DamlDataPayload` which does not distinguish between record and template
    /// types.  To be able to determine which records are templates we need to examine the parent `DamlModulePayload`
    /// to search for a template with a matching name.
    pub fn from_data_wrapper(data: DamlPayloadParentContext<'a>) -> DamlLfConvertResult<Self> {
        match &data.parent {
            DamlPayloadParentContextType::Data(DamlDataPayload::Record(record)) =>
                data.module.template(&record.name).map_or(Ok(DamlDataEnrichedPayload::Record(record)), |template| {
                    Ok(DamlDataEnrichedPayload::Template(template))
                }),
            DamlPayloadParentContextType::Data(DamlDataPayload::Variant(variant)) =>
                Ok(DamlDataEnrichedPayload::Variant(variant)),
            DamlPayloadParentContextType::Data(DamlDataPayload::Enum(data_enum)) =>
                Ok(DamlDataEnrichedPayload::Enum(data_enum)),
            _ => Err(DamlLfConvertError::InternalError(
                "DamlPayloadParentContext.parent not of expected type".to_owned(),
            )),
        }
    }

    /// The name of the data type.
    pub fn name(&self) -> InternableDottedName<'a> {
        match self {
            DamlDataEnrichedPayload::Record(record) => record.name,
            DamlDataEnrichedPayload::Template(template) => template.name,
            DamlDataEnrichedPayload::Variant(variant) => variant.name,
            DamlDataEnrichedPayload::Enum(data_enum) => data_enum.name,
        }
    }
}

#[derive(Debug)]
pub struct DamlTemplatePayload<'a> {
    pub name: InternableDottedName<'a>,
    pub choices: Vec<DamlChoicePayload<'a>>,
    pub param: InternableString<'a>,
    #[cfg(feature = "full")]
    pub precond: Option<DamlExprPayload<'a>>,
    #[cfg(feature = "full")]
    pub signatories: DamlExprPayload<'a>,
    #[cfg(feature = "full")]
    pub agreement: DamlExprPayload<'a>,
    #[cfg(feature = "full")]
    pub observers: DamlExprPayload<'a>,
    pub key: Option<DamlDefKeyPayload<'a>>,
}

impl<'a> DamlTemplatePayload<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: InternableDottedName<'a>,
        choices: Vec<DamlChoicePayload<'a>>,
        param: InternableString<'a>,
        #[cfg(feature = "full")] precond: Option<DamlExprPayload<'a>>,
        #[cfg(feature = "full")] signatories: DamlExprPayload<'a>,
        #[cfg(feature = "full")] agreement: DamlExprPayload<'a>,
        #[cfg(feature = "full")] observers: DamlExprPayload<'a>,
        key: Option<DamlDefKeyPayload<'a>>,
    ) -> Self {
        Self {
            name,
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
            key,
        }
    }
}

impl<'a> TryFrom<&'a DefTemplate> for DamlTemplatePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(def_template: &'a DefTemplate) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            InternableDottedName::from(def_template.tycon.as_ref().req()?),
            def_template.choices.iter().map(DamlChoicePayload::try_from).collect::<DamlLfConvertResult<_>>()?,
            InternableString::from(def_template.param.as_ref().req()?),
            #[cfg(feature = "full")]
            def_template.precond.as_ref().map(DamlExprPayload::try_from).transpose()?,
            #[cfg(feature = "full")]
            DamlExprPayload::try_from(def_template.signatories.as_ref().req()?)?,
            #[cfg(feature = "full")]
            DamlExprPayload::try_from(def_template.agreement.as_ref().req()?)?,
            #[cfg(feature = "full")]
            DamlExprPayload::try_from(def_template.observers.as_ref().req()?)?,
            def_template.key.as_ref().map(DamlDefKeyPayload::try_from).transpose()?,
        ))
    }
}

impl<'a> PartialEq for DamlTemplatePayload<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

///
pub type DamlChoiceWrapper<'a> = PayloadElementWrapper<'a, &'a DamlChoicePayload<'a>>;

#[derive(Debug)]
pub struct DamlChoicePayload<'a> {
    pub consuming: bool,
    pub name: InternableString<'a>,
    pub argument_name: InternableString<'a>,
    pub argument_type: DamlTypePayload<'a>,
    pub return_type: DamlTypePayload<'a>,
    pub self_binder: InternableString<'a>,
    #[cfg(feature = "full")]
    pub update: DamlExprPayload<'a>,
    #[cfg(feature = "full")]
    pub controllers: DamlExprPayload<'a>,
    #[cfg(feature = "full")]
    pub observers: Option<DamlExprPayload<'a>>,
}

impl<'a> DamlChoicePayload<'a> {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        consuming: bool,
        name: InternableString<'a>,
        argument_name: InternableString<'a>,
        argument_type: DamlTypePayload<'a>,
        return_type: DamlTypePayload<'a>,
        self_binder: InternableString<'a>,
        #[cfg(feature = "full")] update: DamlExprPayload<'a>,
        #[cfg(feature = "full")] controllers: DamlExprPayload<'a>,
        #[cfg(feature = "full")] observers: Option<DamlExprPayload<'a>>,
    ) -> Self {
        Self {
            consuming,
            name,
            argument_name,
            argument_type,
            return_type,
            self_binder,
            #[cfg(feature = "full")]
            update,
            #[cfg(feature = "full")]
            controllers,
            #[cfg(feature = "full")]
            observers,
        }
    }
}

impl<'a> TryFrom<&'a TemplateChoice> for DamlChoicePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(template_choice: &'a TemplateChoice) -> DamlLfConvertResult<Self> {
        let (argument_var, argument_type) =
            template_choice.arg_binder.as_ref().req().map(|b| (b.var.as_ref(), b.r#type.as_ref()))?;
        let return_type = template_choice.ret_type.as_ref().req()?;
        #[cfg(feature = "full")]
        let update = template_choice.update.as_ref().req()?;
        #[cfg(feature = "full")]
        let controllers = template_choice.controllers.as_ref().req()?;
        #[cfg(feature = "full")]
        let observers = template_choice.observers.as_ref();
        Ok(Self::new(
            template_choice.consuming,
            InternableString::from(template_choice.name.as_ref().req()?),
            InternableString::from(argument_var.req()?),
            DamlTypePayload::try_from(argument_type.req()?)?,
            DamlTypePayload::try_from(return_type)?,
            InternableString::from(template_choice.self_binder.as_ref().req()?),
            #[cfg(feature = "full")]
            DamlExprPayload::try_from(update)?,
            #[cfg(feature = "full")]
            DamlExprPayload::try_from(controllers)?,
            #[cfg(feature = "full")]
            observers.map(DamlExprPayload::try_from).transpose()?,
        ))
    }
}

pub type DamlDefKeyWrapper<'a> = PayloadElementWrapper<'a, &'a DamlDefKeyPayload<'a>>;

#[derive(Debug)]
pub struct DamlDefKeyPayload<'a> {
    pub ty: DamlTypePayload<'a>,
    #[cfg(feature = "full")]
    pub maintainers: DamlExprPayload<'a>,
    #[cfg(feature = "full")]
    pub key_expr: DamlKeyExprPayload<'a>,
}

impl<'a> DamlDefKeyPayload<'a> {
    pub fn new(
        ty: DamlTypePayload<'a>,
        #[cfg(feature = "full")] maintainers: DamlExprPayload<'a>,
        #[cfg(feature = "full")] key_expr: DamlKeyExprPayload<'a>,
    ) -> Self {
        Self {
            ty,
            #[cfg(feature = "full")]
            maintainers,
            #[cfg(feature = "full")]
            key_expr,
        }
    }
}

impl<'a> TryFrom<&'a DefKey> for DamlDefKeyPayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(def_key: &'a DefKey) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            DamlTypePayload::try_from(def_key.r#type.as_ref().req()?)?,
            #[cfg(feature = "full")]
            DamlExprPayload::try_from(def_key.maintainers.as_ref().req()?)?,
            #[cfg(feature = "full")]
            DamlKeyExprPayload::try_from(def_key.key_expr.as_ref().req()?)?,
        ))
    }
}

#[derive(Debug)]
pub struct DamlRecordPayload<'a> {
    pub name: InternableDottedName<'a>,
    pub fields: Vec<DamlFieldPayload<'a>>,
    pub type_params: Vec<DamlTypeVarWithKindPayload<'a>>,
    pub serializable: bool,
}

impl<'a> DamlRecordPayload<'a> {
    pub fn new(
        name: InternableDottedName<'a>,
        fields: Vec<DamlFieldPayload<'a>>,
        type_params: Vec<DamlTypeVarWithKindPayload<'a>>,
        serializable: bool,
    ) -> Self {
        Self {
            name,
            fields,
            type_params,
            serializable,
        }
    }
}

impl<'a> PartialEq for DamlRecordPayload<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug)]
pub struct DamlVariantPayload<'a> {
    pub name: InternableDottedName<'a>,
    pub fields: Vec<DamlFieldPayload<'a>>,
    pub type_params: Vec<DamlTypeVarWithKindPayload<'a>>,
    pub serializable: bool,
}

impl<'a> DamlVariantPayload<'a> {
    pub fn new(
        name: InternableDottedName<'a>,
        fields: Vec<DamlFieldPayload<'a>>,
        type_params: Vec<DamlTypeVarWithKindPayload<'a>>,
        serializable: bool,
    ) -> Self {
        Self {
            name,
            fields,
            type_params,
            serializable,
        }
    }
}

impl<'a> PartialEq for DamlVariantPayload<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug)]
pub struct DamlEnumPayload<'a> {
    pub name: InternableDottedName<'a>,
    pub constructors_str: &'a [String],
    pub constructors_interned_str: &'a [i32],
    pub type_params: Vec<DamlTypeVarWithKindPayload<'a>>,
    pub serializable: bool,
}

impl<'a> DamlEnumPayload<'a> {
    pub fn new(
        name: InternableDottedName<'a>,
        constructors_str: &'a [String],
        constructors_interned_str: &'a [i32],
        type_params: Vec<DamlTypeVarWithKindPayload<'a>>,
        serializable: bool,
    ) -> Self {
        Self {
            name,
            constructors_str,
            constructors_interned_str,
            type_params,
            serializable,
        }
    }
}

impl<'a> PartialEq for DamlEnumPayload<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
