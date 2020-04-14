use std::collections::HashMap;

use crate::element::daml_package::DamlPackage;
use crate::element::serialize;
use crate::element::visitor::{DamlElementVisitor, DamlVisitableElement};
use itertools::Itertools;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DamlArchive<'a> {
    pub name: &'a str,
    #[serde(serialize_with = "serialize::serialize_map")]
    pub packages: HashMap<&'a str, DamlPackage<'a>>,
}

impl<'a> DamlArchive<'a> {
    pub fn new(name: &'a str, packages: HashMap<&'a str, DamlPackage<'a>>) -> Self {
        Self {
            name,
            packages,
        }
    }
}

impl<'a> DamlVisitableElement<'a> for DamlArchive<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_archive(self);
        if visitor.sort_elements() {
            self.packages.values().sorted_by_key(|p| p.name).for_each(|package| package.accept(visitor));
        } else {
            self.packages.values().for_each(|package| package.accept(visitor));
        }
        visitor.post_visit_archive(self);
    }
}
