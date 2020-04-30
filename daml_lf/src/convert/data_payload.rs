use crate::convert::field_payload::DamlFieldPayload;
use crate::convert::interned::{InternableDottedName, InternableString};
use crate::convert::type_payload::DamlTypePayload;
use crate::convert::typevar_payload::DamlTypeVarPayload;
use crate::convert::util::Required;
use crate::convert::wrapper::{DamlPayloadDataWrapper, PayloadElementWrapper};
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::def_data_type::DataCons;
use crate::lf_protobuf::com::digitalasset::daml_lf_1::{DefDataType, DefTemplate, TemplateChoice};
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
        let type_arguments: Vec<_> =
            def_data_type.params.iter().map(DamlTypeVarPayload::try_from).collect::<DamlLfConvertResult<_>>()?;
        Ok(match def_data_type.data_cons.as_ref().req()? {
            DataCons::Record(fields) => Self::new_record(DamlRecordPayload::new(
                name,
                fields.fields.iter().map(DamlFieldPayload::try_from).collect::<DamlLfConvertResult<_>>()?,
                type_arguments,
            )),
            DataCons::Variant(fields) => Self::new_variant(DamlVariantPayload::new(
                name,
                fields.fields.iter().map(DamlFieldPayload::try_from).collect::<DamlLfConvertResult<_>>()?,
                type_arguments,
            )),
            DataCons::Enum(constructors) => Self::new_enum(DamlEnumPayload::new(
                name,
                constructors.constructors_str.as_slice(),
                constructors.constructors_interned_str.as_slice(),
                type_arguments,
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
    pub fn from_data_wrapper(data: DamlPayloadDataWrapper<'a>) -> Self {
        match data.data {
            DamlDataPayload::Record(record) =>
                if let Some(template) = data.module.template(&record.name) {
                    DamlDataEnrichedPayload::Template(template)
                } else {
                    DamlDataEnrichedPayload::Record(record)
                },
            DamlDataPayload::Variant(variant) => DamlDataEnrichedPayload::Variant(variant),
            DamlDataPayload::Enum(data_enum) => DamlDataEnrichedPayload::Enum(data_enum),
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
}

impl<'a> DamlTemplatePayload<'a> {
    pub fn new(name: InternableDottedName<'a>, choices: Vec<DamlChoicePayload<'a>>) -> Self {
        Self {
            name,
            choices,
        }
    }
}

impl<'a> TryFrom<&'a DefTemplate> for DamlTemplatePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(def_template: &'a DefTemplate) -> DamlLfConvertResult<Self> {
        Ok(Self::new(
            InternableDottedName::from(def_template.tycon.as_ref().req()?),
            def_template.choices.iter().map(DamlChoicePayload::try_from).collect::<DamlLfConvertResult<_>>()?,
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
    pub name: InternableString<'a>,
    pub argument_type: DamlTypePayload<'a>,
    pub return_type: DamlTypePayload<'a>,
}

impl<'a> DamlChoicePayload<'a> {
    pub const fn new(
        name: InternableString<'a>,
        argument_type: DamlTypePayload<'a>,
        return_type: DamlTypePayload<'a>,
    ) -> Self {
        Self {
            name,
            argument_type,
            return_type,
        }
    }
}

impl<'a> TryFrom<&'a TemplateChoice> for DamlChoicePayload<'a> {
    type Error = DamlLfConvertError;

    fn try_from(template_choice: &'a TemplateChoice) -> DamlLfConvertResult<Self> {
        let argument_type = template_choice.arg_binder.as_ref().and_then(|f| f.r#type.as_ref()).req()?;
        let return_type = template_choice.ret_type.as_ref().req()?;
        Ok(Self::new(
            InternableString::from(template_choice.name.as_ref().req()?),
            DamlTypePayload::try_from(argument_type)?,
            DamlTypePayload::try_from(return_type)?,
        ))
    }
}

#[derive(Debug)]
pub struct DamlRecordPayload<'a> {
    pub name: InternableDottedName<'a>,
    pub fields: Vec<DamlFieldPayload<'a>>,
    pub type_arguments: Vec<DamlTypeVarPayload<'a>>,
}

impl<'a> DamlRecordPayload<'a> {
    pub fn new(
        name: InternableDottedName<'a>,
        fields: Vec<DamlFieldPayload<'a>>,
        type_arguments: Vec<DamlTypeVarPayload<'a>>,
    ) -> Self {
        Self {
            name,
            fields,
            type_arguments,
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
    pub type_arguments: Vec<DamlTypeVarPayload<'a>>,
}

impl<'a> DamlVariantPayload<'a> {
    pub fn new(
        name: InternableDottedName<'a>,
        fields: Vec<DamlFieldPayload<'a>>,
        type_arguments: Vec<DamlTypeVarPayload<'a>>,
    ) -> Self {
        Self {
            name,
            fields,
            type_arguments,
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
    pub type_arguments: Vec<DamlTypeVarPayload<'a>>,
}

impl<'a> DamlEnumPayload<'a> {
    pub fn new(
        name: InternableDottedName<'a>,
        constructors_str: &'a [String],
        constructors_interned_str: &'a [i32],
        type_arguments: Vec<DamlTypeVarPayload<'a>>,
    ) -> Self {
        Self {
            name,
            constructors_str,
            constructors_interned_str,
            type_arguments,
        }
    }
}

impl<'a> PartialEq for DamlEnumPayload<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
