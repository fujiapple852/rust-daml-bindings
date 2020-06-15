use crate::element::daml_data::DamlData;
use crate::element::visitor::DamlElementVisitor;
#[cfg(feature = "full")]
use crate::element::DamlDefValue;
use crate::element::DamlVisitableElement;
use crate::element::{serialize, DamlType, DamlTypeVarWithKind};
use itertools::Itertools;
use serde::Serialize;
use std::collections::HashMap;

const ROOT_MODULE_NAME: &str = "root";

#[derive(Debug, Serialize, Clone)]
pub struct DamlModule<'a> {
    path: Vec<&'a str>,
    flags: DamlFeatureFlags,
    synonyms: Vec<DamlDefTypeSyn<'a>>,
    #[serde(serialize_with = "serialize::serialize_map")]
    child_modules: HashMap<&'a str, DamlModule<'a>>,
    #[serde(serialize_with = "serialize::serialize_map")]
    data_types: HashMap<&'a str, DamlData<'a>>,
    #[cfg(feature = "full")]
    values: Vec<DamlDefValue<'a>>,
}

impl<'a> DamlModule<'a> {
    /// Create an empty `DamlModule` with a given `path`.
    pub fn new_empty(path: Vec<&'a str>) -> Self {
        Self {
            path,
            flags: DamlFeatureFlags::default(),
            synonyms: Vec::default(),
            child_modules: HashMap::default(),
            data_types: HashMap::default(),
            #[cfg(feature = "full")]
            values: Vec::default(),
        }
    }

    /// Create a leaf `DamlModule`.
    pub fn new_leaf(
        path: Vec<&'a str>,
        flags: DamlFeatureFlags,
        synonyms: Vec<DamlDefTypeSyn<'a>>,
        data_types: HashMap<&'a str, DamlData<'a>>,
        #[cfg(feature = "full")] values: Vec<DamlDefValue<'a>>,
    ) -> Self {
        Self {
            path,
            flags,
            synonyms,
            child_modules: HashMap::default(),
            data_types,
            #[cfg(feature = "full")]
            values,
        }
    }

    /// Populate this module with data taken from another module.
    pub fn take_from(&mut self, other: Self) {
        debug_assert_eq!(self.path, other.path);
        self.flags = other.flags;
        self.data_types = other.data_types;
        self.synonyms = other.synonyms;
        #[cfg(feature = "full")]
        {
            self.values = other.values;
        }
    }

    /// Create a root `DamlModule`.
    pub fn new_root() -> Self {
        Self::new_empty(vec![])
    }

    /// The `DamlFeatureFlags` of the module.
    pub fn flags(&self) -> &DamlFeatureFlags {
        &self.flags
    }

    /// The `DamlDefTypeSyn` of the module.
    pub fn synonyms(&self) -> &[DamlDefTypeSyn<'_>] {
        &self.synonyms
    }

    /// The module path.
    pub fn path(&self) -> &[&str] {
        &self.path
    }

    /// The child `DamlModule` of the module.
    pub fn child_modules(&self) -> &HashMap<&'a str, DamlModule<'a>> {
        &self.child_modules
    }

    pub fn child_modules_mut(&mut self) -> &mut HashMap<&'a str, DamlModule<'a>> {
        &mut self.child_modules
    }

    /// The `DamlData` declared in the module.
    pub fn data_types(&self) -> &HashMap<&'a str, DamlData<'a>> {
        &self.data_types
    }

    /// The `DamlDefValue` declared in the module.
    #[cfg(feature = "full")]
    pub fn values(&self) -> &[DamlDefValue<'a>] {
        &self.values
    }

    /// Returns `true` if this is a root module, `false` otherwise.
    pub fn is_root(&self) -> bool {
        self.path.is_empty()
    }

    /// Returns `true` if this is a leaf module, `false` otherwise.
    pub fn is_leaf(&self) -> bool {
        self.child_modules.is_empty()
    }

    /// Returns the local name of the module.
    pub fn local_name(&self) -> &str {
        self.path.last().unwrap_or_else(|| &ROOT_MODULE_NAME)
    }

    /// Retrieve a child model by relative path or `None` if no such module exists.
    pub fn child_module(&self, name: &str) -> Option<&DamlModule<'_>> {
        self.child_modules.get(name)
    }

    /// Retrieve a child `DamlModule` by relative path or `None` if no such module exists.
    pub fn child_module_path<'b, S: AsRef<str>>(&'a self, relative_path: &'b [S]) -> Option<&'a DamlModule<'a>> {
        match relative_path {
            [] => Some(self),
            [head, tail @ ..] => match self.child_module(head.as_ref()) {
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

/// Definition of a DAML Type Synonym.
#[derive(Debug, Serialize, Clone)]
pub struct DamlDefTypeSyn<'a> {
    params: Vec<DamlTypeVarWithKind<'a>>,
    ty: DamlType<'a>,
    name: Vec<&'a str>,
}

impl<'a> DamlDefTypeSyn<'a> {
    /// Create a type synonym.
    pub fn new(params: Vec<DamlTypeVarWithKind<'a>>, ty: DamlType<'a>, name: Vec<&'a str>) -> Self {
        Self {
            params,
            ty,
            name,
        }
    }

    ///
    pub fn params(&self) -> &[DamlTypeVarWithKind<'_>] {
        &self.params
    }

    /// Type of the defined type synonym.
    pub fn ty(&self) -> &DamlType<'_> {
        &self.ty
    }

    /// Name of the defined type synonym.
    pub fn name(&self) -> &[&str] {
        &self.name
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

/// DOCME
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

    ///
    pub fn forbid_party_literals(&self) -> bool {
        self.forbid_party_literals
    }

    ///
    pub fn dont_divulge_contract_ids_in_create_arguments(&self) -> bool {
        self.dont_divulge_contract_ids_in_create_arguments
    }

    ///
    pub fn dont_disclose_non_consuming_choices_to_observers(&self) -> bool {
        self.dont_disclose_non_consuming_choices_to_observers
    }
}
