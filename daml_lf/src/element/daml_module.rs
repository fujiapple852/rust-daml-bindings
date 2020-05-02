use std::collections::HashMap;

use crate::element::daml_data::DamlData;
use crate::element::serialize;
use crate::element::visitor::DamlElementVisitor;
use crate::element::DamlVisitableElement;
use itertools::Itertools;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DamlModule<'a> {
    pub path: Vec<&'a str>,
    #[serde(serialize_with = "serialize::serialize_map")]
    pub child_modules: HashMap<&'a str, DamlModule<'a>>,
    #[serde(serialize_with = "serialize::serialize_map")]
    pub data_types: HashMap<&'a str, DamlData<'a>>,
}

impl<'a> DamlModule<'a> {
    pub fn new(path: Vec<&'a str>) -> Self {
        Self {
            path,
            child_modules: HashMap::default(),
            data_types: HashMap::default(),
        }
    }

    pub fn new_root() -> Self {
        Self::new(vec![])
    }

    pub fn is_root(&self) -> bool {
        self.path.is_empty()
    }

    pub fn name(&self) -> &str {
        self.path.last().unwrap_or_else(|| &"root")
    }

    /// Retrieve a child model by relative path or `None` if no such module exists.
    pub fn child_module<'b>(&'a self, relative_path: &'b [&'a str]) -> Option<&'a DamlModule<'a>> {
        match relative_path {
            [] => Some(self),
            [head, tail @ ..] => self.child_modules.get(head).and_then(|m| m.child_module(tail)),
        }
    }
}

impl<'a> DamlVisitableElement<'a> for DamlModule<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_module(self);
        if visitor.sort_elements() {
            self.data_types.values().sorted_by_key(|ty| ty.name()).for_each(|data| data.accept(visitor));
            self.child_modules.values().sorted_by_key(|&m| m.path.clone()).for_each(|module| module.accept(visitor));
        } else {
            self.data_types.values().for_each(|data| data.accept(visitor));
            self.child_modules.values().for_each(|module| module.accept(visitor));
        }
        visitor.post_visit_module(self);
    }
}
