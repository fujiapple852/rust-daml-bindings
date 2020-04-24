use crate::element::daml_type::DamlType;
use crate::element::visitor::DamlElementVisitor;
use crate::element::DamlVisitableElement;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DamlField<'a> {
    pub name: &'a str,
    pub ty: DamlType<'a>,
}

impl<'a> DamlField<'a> {
    pub const fn new(name: &'a str, ty: DamlType<'a>) -> Self {
        Self {
            name,
            ty,
        }
    }
}

impl<'a> DamlVisitableElement<'a> for DamlField<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_field(self);
        self.ty.accept(visitor);
        visitor.post_visit_field(self);
    }
}
