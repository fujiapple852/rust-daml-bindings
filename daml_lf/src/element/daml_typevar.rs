use crate::element::visitor::DamlElementVisitor;
use crate::element::DamlVisitableElement;
use bounded_static::ToBoundedStatic;
use serde::Serialize;
use std::borrow::Cow;

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlTypeVarWithKind<'a> {
    var: Cow<'a, str>,
    kind: DamlKind,
}

impl<'a> DamlTypeVarWithKind<'a> {
    pub const fn new(var: Cow<'a, str>, kind: DamlKind) -> Self {
        Self {
            var,
            kind,
        }
    }

    pub fn var(&self) -> &str {
        &self.var
    }

    pub const fn kind(&self) -> &DamlKind {
        &self.kind
    }
}

impl<'a> DamlVisitableElement<'a> for DamlTypeVarWithKind<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_type_var(self);
        self.kind.accept(visitor);
        visitor.post_visit_type_var(self);
    }
}

impl ToBoundedStatic for DamlTypeVarWithKind<'_> {
    type Static = DamlTypeVarWithKind<'static>;

    fn to_static(&self) -> Self::Static {
        DamlTypeVarWithKind::new(self.var.to_static(), self.kind.clone())
    }
}

/// DOCME
#[derive(Debug, Serialize, Clone)]
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

/// DOCME
#[derive(Debug, Serialize, Clone)]
pub struct DamlArrow {
    params: Vec<DamlKind>,
    result: DamlKind,
}

impl DamlArrow {
    pub fn new(params: Vec<DamlKind>, result: DamlKind) -> Self {
        Self {
            params,
            result,
        }
    }

    pub fn params(&self) -> &[DamlKind] {
        &self.params
    }

    pub const fn result(&self) -> &DamlKind {
        &self.result
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
