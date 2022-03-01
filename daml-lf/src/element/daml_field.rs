use crate::element::daml_type::DamlType;
use crate::element::visitor::DamlElementVisitor;
use crate::element::DamlVisitableElement;
use bounded_static::ToStatic;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlField<'a> {
    name: Cow<'a, str>,
    ty: DamlType<'a>,
}

impl<'a> DamlField<'a> {
    pub const fn new(name: Cow<'a, str>, ty: DamlType<'a>) -> Self {
        Self {
            name,
            ty,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn ty(&self) -> &DamlType<'a> {
        &self.ty
    }
}

impl<'a> DamlVisitableElement<'a> for DamlField<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_field(self);
        self.ty.accept(visitor);
        visitor.post_visit_field(self);
    }
}
