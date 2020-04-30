use crate::element::visitor::DamlElementVisitor;
use crate::element::DamlVisitableElement;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DamlTypeVar<'a> {
    pub var: &'a str,
    pub kind: DamlKind,
}

impl<'a> DamlTypeVar<'a> {
    pub const fn new(var: &'a str, kind: DamlKind) -> Self {
        Self {
            var,
            kind,
        }
    }
}

impl<'a> DamlVisitableElement<'a> for DamlTypeVar<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_type_var(self);
        self.kind.accept(visitor);
        visitor.post_visit_type_var(self);
    }
}

#[derive(Debug, Serialize)]
pub enum DamlKind {
    Star,
    Arrow(Box<DamlArrow>),
    Nat,
}

impl DamlVisitableElement<'_> for DamlKind {
    fn accept<'a>(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_kind(self);
        if let DamlKind::Arrow(kind) = self {
            kind.accept(visitor);
        }
        visitor.post_visit_kind(self);
    }
}

#[derive(Debug, Serialize)]
pub struct DamlArrow {
    pub params: Vec<DamlKind>,
    pub result: DamlKind,
}

impl DamlArrow {
    pub fn new(params: Vec<DamlKind>, result: DamlKind) -> Self {
        Self {
            params,
            result,
        }
    }
}

impl DamlVisitableElement<'_> for DamlArrow {
    fn accept<'a>(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_arrow(self);
        self.params.iter().for_each(|param| param.accept(visitor));
        self.result.accept(visitor);
        visitor.post_visit_arrow(self);
    }
}
