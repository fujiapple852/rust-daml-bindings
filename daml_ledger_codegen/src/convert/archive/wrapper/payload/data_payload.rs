use crate::convert::archive::wrapper::payload::*;
use daml_lf::protobuf_autogen::daml_lf_1::def_data_type::*;
use daml_lf::protobuf_autogen::daml_lf_1::*;

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

impl<'a> From<&'a DefDataType> for DamlDataPayload<'a> {
    fn from(def_data_type: &'a DefDataType) -> Self {
        let name = InternableDottedName::from(def_data_type.name.as_ref().expect("DefDataType.name"));
        match def_data_type.data_cons.as_ref().expect("DefDataType.data_cons") {
            DataCons::Record(fields) =>
                Self::new_record(RecordPayload::new(name, fields.fields.iter().map(DamlFieldPayload::from).collect())),
            DataCons::Variant(fields) =>
                Self::new_variant(VariantPayload::new(name, fields.fields.iter().map(DamlFieldPayload::from).collect())),
            DataCons::Enum(constructors) => Self::new_enum(EnumPayload::new(
                name,
                constructors.constructors_str.as_slice(),
                constructors.constructors_interned_str.as_slice(),
            )),
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

impl<'a> From<&'a DefTemplate> for DamlTemplatePayload<'a> {
    fn from(def_template: &'a DefTemplate) -> Self {
        Self::new(
            InternableDottedName::from(def_template.tycon.as_ref().expect("DefTemplate.tycon")),
            def_template.choices.iter().map(DamlChoicePayload::from).collect(),
        )
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

impl<'a> From<&'a TemplateChoice> for DamlChoicePayload<'a> {
    fn from(template_choice: &'a TemplateChoice) -> Self {
        let argument_type =
            template_choice.arg_binder.as_ref().and_then(|f| f.r#type.as_ref()).expect("TemplateChoice.arg_binder");
        let return_type = template_choice.ret_type.as_ref().expect("TemplateChoice.ret_type");
        Self::new(
            InternableString::from(template_choice.name.as_ref().expect("TemplateChoice.name")),
            DamlTypePayload::from(argument_type),
            DamlTypePayload::from(return_type),
        )
    }
}

#[derive(Debug)]
pub struct RecordPayload<'a> {
    pub name: InternableDottedName<'a>,
    pub fields: Vec<DamlFieldPayload<'a>>,
}

impl<'a> RecordPayload<'a> {
    pub fn new(name: InternableDottedName<'a>, fields: Vec<DamlFieldPayload<'a>>) -> Self {
        Self {
            name,
            fields,
        }
    }
}

#[derive(Debug)]
pub struct VariantPayload<'a> {
    pub name: InternableDottedName<'a>,
    pub fields: Vec<DamlFieldPayload<'a>>,
}

impl<'a> VariantPayload<'a> {
    pub fn new(name: InternableDottedName<'a>, fields: Vec<DamlFieldPayload<'a>>) -> Self {
        Self {
            name,
            fields,
        }
    }
}

#[derive(Debug)]
pub struct EnumPayload<'a> {
    pub name: InternableDottedName<'a>,
    pub constructors_str: &'a [String],
    pub constructors_interned_str: &'a [i32],
}

impl<'a> EnumPayload<'a> {
    pub fn new(
        name: InternableDottedName<'a>,
        constructors_str: &'a [String],
        constructors_interned_str: &'a [i32],
    ) -> Self {
        Self {
            name,
            constructors_str,
            constructors_interned_str,
        }
    }
}
