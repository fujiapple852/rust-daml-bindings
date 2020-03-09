use crate::convert::archive::wrapper::payload::util::Required;
use crate::convert::archive::wrapper::payload::{
    DamlFieldPayload, DamlTypePayload, DamlTypeVarPayload, InternableDottedName, InternableString,
};
use crate::convert::error::{DamlCodeGenError, DamlCodeGenResult};
use daml_lf::protobuf_autogen::daml_lf_1::def_data_type::DataCons;
use daml_lf::protobuf_autogen::daml_lf_1::{DefDataType, DefTemplate, TemplateChoice};
use std::convert::TryFrom;

#[derive(Debug)]
pub enum DamlDataPayload<'a> {
    Record(RecordPayload<'a>),
    Variant(VariantPayload<'a>),
    Enum(EnumPayload<'a>),
}

impl<'a> PartialEq for DamlDataPayload<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl<'a> DamlDataPayload<'a> {
    pub fn new_record(daml_record: impl Into<RecordPayload<'a>>) -> Self {
        DamlDataPayload::Record(daml_record.into())
    }

    pub fn new_variant(daml_variant: impl Into<VariantPayload<'a>>) -> Self {
        DamlDataPayload::Variant(daml_variant.into())
    }

    pub fn new_enum(daml_enum: impl Into<EnumPayload<'a>>) -> Self {
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
    type Error = DamlCodeGenError;

    fn try_from(def_data_type: &'a DefDataType) -> DamlCodeGenResult<Self> {
        let name = InternableDottedName::from(def_data_type.name.as_ref().req()?);
        let type_arguments: Vec<_> =
            def_data_type.params.iter().map(DamlTypeVarPayload::try_from).collect::<DamlCodeGenResult<_>>()?;
        Ok(match def_data_type.data_cons.as_ref().req()? {
            DataCons::Record(fields) => Self::new_record(RecordPayload::new(
                name,
                fields.fields.iter().map(DamlFieldPayload::try_from).collect::<DamlCodeGenResult<_>>()?,
                type_arguments,
            )),
            DataCons::Variant(fields) => Self::new_variant(VariantPayload::new(
                name,
                fields.fields.iter().map(DamlFieldPayload::try_from).collect::<DamlCodeGenResult<_>>()?,
                type_arguments,
            )),
            DataCons::Enum(constructors) => Self::new_enum(EnumPayload::new(
                name,
                constructors.constructors_str.as_slice(),
                constructors.constructors_interned_str.as_slice(),
                type_arguments,
            )),
        })
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
    type Error = DamlCodeGenError;

    fn try_from(def_template: &'a DefTemplate) -> DamlCodeGenResult<Self> {
        Ok(Self::new(
            InternableDottedName::from(def_template.tycon.as_ref().req()?),
            def_template.choices.iter().map(DamlChoicePayload::try_from).collect::<DamlCodeGenResult<_>>()?,
        ))
    }
}

#[derive(Debug)]
pub struct DamlChoicePayload<'a> {
    pub name: InternableString<'a>,
    pub argument_type: DamlTypePayload<'a>,
    pub return_type: DamlTypePayload<'a>,
}

impl<'a> DamlChoicePayload<'a> {
    pub fn new(
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
    type Error = DamlCodeGenError;

    fn try_from(template_choice: &'a TemplateChoice) -> DamlCodeGenResult<Self> {
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
pub struct RecordPayload<'a> {
    pub name: InternableDottedName<'a>,
    pub fields: Vec<DamlFieldPayload<'a>>,
    pub type_arguments: Vec<DamlTypeVarPayload<'a>>,
}

impl<'a> RecordPayload<'a> {
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

#[derive(Debug)]
pub struct VariantPayload<'a> {
    pub name: InternableDottedName<'a>,
    pub fields: Vec<DamlFieldPayload<'a>>,
    pub type_arguments: Vec<DamlTypeVarPayload<'a>>,
}

impl<'a> VariantPayload<'a> {
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

#[derive(Debug)]
pub struct EnumPayload<'a> {
    pub name: InternableDottedName<'a>,
    pub constructors_str: &'a [String],
    pub constructors_interned_str: &'a [i32],
    pub type_arguments: Vec<DamlTypeVarPayload<'a>>,
}

impl<'a> EnumPayload<'a> {
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
