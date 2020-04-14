use crate::element::daml_field::DamlField;
use crate::element::visitor::DamlElementVisitor;
use crate::element::{DamlType, DamlTypeVar, DamlVisitableElement};
use serde::Serialize;

#[derive(Debug, Serialize)]
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

impl<'a> DamlVisitableElement<'a> for DamlData<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_data(self);
        match self {
            DamlData::Record(record) => record.accept(visitor),
            DamlData::Template(template) => template.accept(visitor),
            DamlData::Variant(variant) => variant.accept(visitor),
            DamlData::Enum(data_enum) => data_enum.accept(visitor),
        }
        visitor.post_visit_data(self);
    }
}

#[derive(Debug, Serialize)]
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

impl<'a> DamlVisitableElement<'a> for DamlTemplate<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_template(self);
        self.fields.iter().for_each(|field| field.accept(visitor));
        self.choices.iter().for_each(|choice| choice.accept(visitor));
        visitor.post_visit_template(self);
    }
}

#[derive(Debug, Serialize)]
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

impl<'a> DamlVisitableElement<'a> for DamlChoice<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_choice(self);
        self.fields.iter().for_each(|field| field.accept(visitor));
        self.return_type.accept(visitor);
        visitor.post_visit_choice(self);
    }
}

#[derive(Debug, Serialize)]
pub struct DamlRecord<'a> {
    pub name: &'a str,
    pub fields: Vec<DamlField<'a>>,
    pub type_arguments: Vec<DamlTypeVar<'a>>,
}

impl<'a> DamlRecord<'a> {
    pub fn new(name: &'a str, fields: Vec<DamlField<'a>>, type_arguments: Vec<DamlTypeVar<'a>>) -> Self {
        Self {
            name,
            fields,
            type_arguments,
        }
    }
}

impl<'a> DamlVisitableElement<'a> for DamlRecord<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_record(self);
        self.fields.iter().for_each(|field| field.accept(visitor));
        self.type_arguments.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_record(self);
    }
}

#[derive(Debug, Serialize)]
pub struct DamlVariant<'a> {
    pub name: &'a str,
    pub fields: Vec<DamlField<'a>>,
    pub type_arguments: Vec<DamlTypeVar<'a>>,
}

impl<'a> DamlVariant<'a> {
    pub fn new(name: &'a str, fields: Vec<DamlField<'a>>, type_arguments: Vec<DamlTypeVar<'a>>) -> Self {
        Self {
            name,
            fields,
            type_arguments,
        }
    }
}

impl<'a> DamlVisitableElement<'a> for DamlVariant<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_variant(self);
        self.fields.iter().for_each(|field| field.accept(visitor));
        self.type_arguments.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_variant(self);
    }
}

#[derive(Debug, Serialize)]
pub struct DamlEnum<'a> {
    pub name: &'a str,
    pub constructors: Vec<&'a str>,
    pub type_arguments: Vec<DamlTypeVar<'a>>,
}

impl<'a> DamlEnum<'a> {
    pub fn new(name: &'a str, constructors: Vec<&'a str>, type_arguments: Vec<DamlTypeVar<'a>>) -> Self {
        Self {
            name,
            constructors,
            type_arguments,
        }
    }
}

impl<'a> DamlVisitableElement<'a> for DamlEnum<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_enum(self);
        self.type_arguments.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_enum(self);
    }
}
