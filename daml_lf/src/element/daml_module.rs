use crate::element::daml_data::DamlData;
use crate::element::visitor::DamlElementVisitor;
#[cfg(feature = "full")]
use crate::element::DamlDefValue;
use crate::element::DamlVisitableElement;
use crate::element::{serialize, DamlType, DamlTypeVarWithKind};
use itertools::Itertools;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone)]
pub struct DamlModule<'a> {
    flags: DamlFeatureFlags,
    path: Vec<&'a str>,
    synonyms: Vec<DamlDefTypeSyn<'a>>,
    #[serde(serialize_with = "serialize::serialize_map")]
    child_modules: HashMap<&'a str, DamlModule<'a>>,
    #[serde(serialize_with = "serialize::serialize_map")]
    data_types: HashMap<&'a str, DamlData<'a>>,
    #[cfg(feature = "full")]
    values: Vec<DamlDefValue<'a>>,
}

impl<'a> DamlModule<'a> {
    pub fn new(path: Vec<&'a str>) -> Self {
        Self {
            flags: DamlFeatureFlags::default(),
            synonyms: Vec::default(),
            path,
            child_modules: HashMap::default(),
            data_types: HashMap::default(),
            #[cfg(feature = "full")]
            values: Vec::default(),
        }
    }

    pub fn new_root() -> Self {
        Self::new(vec![])
    }

    pub fn flags(&self) -> &DamlFeatureFlags {
        &self.flags
    }

    pub fn synonyms(&self) -> &[DamlDefTypeSyn<'_>] {
        &self.synonyms
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

    #[cfg(feature = "full")]
    pub fn values(&self) -> &[DamlDefValue<'a>] {
        &self.values
    }

    pub fn update_from_parts(
        &mut self,
        flags: DamlFeatureFlags,
        synonyms: Vec<DamlDefTypeSyn<'a>>,
        data_types: HashMap<&'a str, DamlData<'a>>,
        #[cfg(feature = "full")] values: Vec<DamlDefValue<'a>>,
    ) {
        self.flags = flags;
        self.synonyms = synonyms;
        self.data_types = data_types;
        #[cfg(feature = "full")]
        {
            self.values = values;
        }
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
        self.synonyms.iter().for_each(|syn| syn.accept(visitor));
        if visitor.sort_elements() {
            self.data_types.values().sorted_by_key(|ty| ty.name()).for_each(|data| data.accept(visitor));
            self.child_modules.values().sorted_by_key(|&m| m.path.clone()).for_each(|module| module.accept(visitor));
        } else {
            self.data_types.values().for_each(|data| data.accept(visitor));
            self.child_modules.values().for_each(|module| module.accept(visitor));
        }
        #[cfg(feature = "full")]
        self.values.iter().for_each(|value| value.accept(visitor));
        visitor.post_visit_module(self);
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlDefTypeSyn<'a> {
    params: Vec<DamlTypeVarWithKind<'a>>,
    ty: DamlType<'a>,
    name: Vec<&'a str>,
}

impl<'a> DamlDefTypeSyn<'a> {
    pub fn new(params: Vec<DamlTypeVarWithKind<'a>>, ty: DamlType<'a>, name: Vec<&'a str>) -> Self {
        Self {
            params,
            ty,
            name,
        }
    }
}

impl<'a> DamlVisitableElement<'a> for DamlDefTypeSyn<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_def_type_syn(self);
        self.params.iter().for_each(|param| param.accept(visitor));
        self.ty.accept(visitor);
        visitor.post_visit_def_type_syn(self);
    }
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct DamlFeatureFlags {
    forbid_party_literals: bool,
    dont_divulge_contract_ids_in_create_arguments: bool,
    dont_disclose_non_consuming_choices_to_observers: bool,
}

impl DamlFeatureFlags {
    pub fn new(
        forbid_party_literals: bool,
        dont_divulge_contract_ids_in_create_arguments: bool,
        dont_disclose_non_consuming_choices_to_observers: bool,
    ) -> Self {
        Self {
            forbid_party_literals,
            dont_divulge_contract_ids_in_create_arguments,
            dont_disclose_non_consuming_choices_to_observers,
        }
    }
}
