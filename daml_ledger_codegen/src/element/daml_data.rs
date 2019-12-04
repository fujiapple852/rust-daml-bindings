use crate::element::daml_field::DamlField;
use crate::element::DamlType;

#[derive(Debug)]
pub enum DamlData<'a> {
    Template(DamlTemplate<'a>),
    Record(DamlRecord<'a>),
    Variant(DamlVariant<'a>),
    Enum(DamlEnum<'a>),
}

impl<'a> DamlData<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            DamlData::Record(record) => record.name,
            DamlData::Template(template) => template.name,
            DamlData::Variant(variant) => variant.name,
            DamlData::Enum(data_enum) => data_enum.name,
        }
    }
}

#[derive(Debug)]
pub struct DamlTemplate<'a> {
    pub name: &'a str,
    pub package_id: &'a str,
    pub module_path: Vec<&'a str>,
    pub fields: Vec<DamlField<'a>>,
    pub choices: Vec<DamlChoice<'a>>,
}

impl<'a> DamlTemplate<'a> {
    pub fn new(
        name: &'a str,
        package_id: &'a str,
        module_path: Vec<&'a str>,
        fields: Vec<DamlField<'a>>,
        choices: Vec<DamlChoice<'a>>,
    ) -> Self {
        Self {
            name,
            package_id,
            module_path,
            fields,
            choices,
        }
    }
}

#[derive(Debug)]
pub struct DamlChoice<'a> {
    pub name: &'a str,
    pub fields: Vec<DamlField<'a>>,
    pub return_type: DamlType<'a>,
}

impl<'a> DamlChoice<'a> {
    pub fn new(name: &'a str, fields: Vec<DamlField<'a>>, return_type: DamlType<'a>) -> Self {
        Self {
            name,
            fields,
            return_type,
        }
    }
}

#[derive(Debug)]
pub struct DamlRecord<'a> {
    pub name: &'a str,
    pub fields: Vec<DamlField<'a>>,
}

impl<'a> DamlRecord<'a> {
    pub fn new(name: &'a str, fields: Vec<DamlField<'a>>) -> Self {
        Self {
            name,
            fields,
        }
    }
}

#[derive(Debug)]
pub struct DamlVariant<'a> {
    pub name: &'a str,
    pub fields: Vec<DamlField<'a>>,
}

impl<'a> DamlVariant<'a> {
    pub fn new(name: &'a str, fields: Vec<DamlField<'a>>) -> Self {
        Self {
            name,
            fields,
        }
    }
}

#[derive(Debug)]
pub struct DamlEnum<'a> {
    pub name: &'a str,
    pub constructors: Vec<&'a str>,
}

impl<'a> DamlEnum<'a> {
    pub fn new(name: &'a str, constructors: Vec<&'a str>) -> Self {
        Self {
            name,
            constructors,
        }
    }
}
