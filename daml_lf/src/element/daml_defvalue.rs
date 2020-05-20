use crate::element::daml_expr::DamlExpr;
use crate::element::{DamlElementVisitor, DamlType, DamlVisitableElement};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct DamlDefValue<'a> {
    pub name: Vec<&'a str>,
    pub ty: DamlType<'a>,
    pub expr: DamlExpr<'a>,
    pub no_party_literals: bool,
    pub is_test: bool,
}

impl<'a> DamlDefValue<'a> {
    pub const fn new(
        name: Vec<&'a str>,
        ty: DamlType<'a>,
        expr: DamlExpr<'a>,
        no_party_literals: bool,
        is_test: bool,
    ) -> Self {
        Self {
            name,
            ty,
            expr,
            no_party_literals,
            is_test,
        }
    }

    pub fn name(&self) -> &[&str] {
        &self.name
    }

    pub const fn ty(&self) -> &DamlType<'a> {
        &self.ty
    }

    pub const fn expr(&self) -> &DamlExpr<'a> {
        &self.expr
    }

    pub const fn no_party_literals(&self) -> bool {
        self.no_party_literals
    }

    pub const fn is_test(&self) -> bool {
        self.is_test
    }
}

impl<'a> DamlVisitableElement<'a> for DamlDefValue<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_def_value(self);
        self.ty.accept(visitor);
        self.expr.accept(visitor);
        visitor.post_visit_def_value(self);
    }
}
