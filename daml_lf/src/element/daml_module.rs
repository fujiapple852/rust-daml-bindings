use std::collections::HashMap;

use crate::element::daml_data::DamlData;
use crate::element::serialize;
use crate::element::visitor::DamlElementVisitor;
use crate::element::DamlVisitableElement;
use itertools::Itertools;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct DamlModule<'a> {
    path: Vec<&'a str>,
    #[serde(serialize_with = "serialize::serialize_map")]
    child_modules: HashMap<&'a str, DamlModule<'a>>,
    #[serde(serialize_with = "serialize::serialize_map")]
    data_types: HashMap<&'a str, DamlData<'a>>,
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

    pub fn path(&self) -> &[&str] {
        &self.path
    }

    pub fn child_modules(&self) -> &HashMap<&'a str, DamlModule<'a>> {
        &self.child_modules
    }

    pub fn child_modules_mut(&mut self) -> &mut HashMap<&'a str, DamlModule<'a>> {
        &mut self.child_modules
    }

    pub fn data_types(&self) -> &HashMap<&'a str, DamlData<'a>> {
        &self.data_types
    }

    pub fn set_data_types(&mut self, data_types: HashMap<&'a str, DamlData<'a>>) {
        self.data_types = data_types;
    }

    pub fn is_root(&self) -> bool {
        self.path.is_empty()
    }

    pub fn name(&self) -> &str {
        self.path.last().unwrap_or_else(|| &"root")
    }

    /// Return the immediate child module `name` or `None` if no such module exists.
    pub fn child_module(&self, name: &str) -> Option<&DamlModule<'_>> {
        self.child_modules.get(name)
    }

    /// Retrieve a child `DamlModule` by relative path or `None` if no such module exists.
    pub fn child_module_path<'b>(&'a self, relative_path: &'b [&'b str]) -> Option<&'a DamlModule<'a>> {
        match relative_path {
            [] => Some(self),
            [head, tail @ ..] => match self.child_module(head) {
                Some(m) => m.child_module_path(tail),
                None => None,
            },
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
