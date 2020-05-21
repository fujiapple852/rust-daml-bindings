use crate::element::daml_data::DamlData;
use crate::element::visitor::DamlElementVisitor;
#[cfg(feature = "full")]
use crate::element::DamlDefValue;
#[cfg(feature = "full")]
use crate::element::DamlExpr;
use crate::element::DamlVisitableElement;
use crate::element::{serialize, DamlType, DamlTypeVarWithKind};
use crate::owned::ToStatic;
use itertools::Itertools;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::once;

const ROOT_MODULE_NAME: &str = "root";

#[derive(Debug, Serialize, Clone)]
pub struct DamlModule<'a> {
    path: Vec<Cow<'a, str>>,
    flags: DamlFeatureFlags,
    synonyms: Vec<DamlDefTypeSyn<'a>>,
    #[serde(serialize_with = "serialize::serialize_map")]
    child_modules: HashMap<Cow<'a, str>, DamlModule<'a>>,
    #[serde(serialize_with = "serialize::serialize_map")]
    data_types: HashMap<Cow<'a, str>, DamlData<'a>>,
    exceptions: Vec<DamlDefException<'a>>,
    #[cfg(feature = "full")]
    values: Vec<DamlDefValue<'a>>,
}

impl<'a> DamlModule<'a> {
    /// Create a root `DamlModule`.
    pub fn new_root() -> Self {
        Self::new_empty(vec![])
    }

    /// Create a leaf `DamlModule`.
    pub fn new_leaf(
        path: Vec<Cow<'a, str>>,
        flags: DamlFeatureFlags,
        synonyms: Vec<DamlDefTypeSyn<'a>>,
        data_types: HashMap<Cow<'a, str>, DamlData<'a>>,
        exceptions: Vec<DamlDefException<'a>>,
        #[cfg(feature = "full")] values: Vec<DamlDefValue<'a>>,
    ) -> Self {
        Self {
            path,
            flags,
            synonyms,
            child_modules: HashMap::default(),
            data_types,
            exceptions,
            #[cfg(feature = "full")]
            values,
        }
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
    pub fn path(&self) -> impl Iterator<Item = &str> {
        self.path.iter().map(AsRef::as_ref)
    }

    /// The child `DamlModule` of the module.
    pub fn child_modules(&self) -> impl Iterator<Item = &DamlModule<'a>> {
        self.child_modules.values()
    }

    /// The `DamlData` declared in the module.
    pub fn data_types(&self) -> impl Iterator<Item = &DamlData<'a>> {
        self.data_types.values()
    }

    /// The `DamlDefException` declared in the module.
    pub fn exceptions(&self) -> &[DamlDefException<'_>] {
        &self.exceptions
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
        self.path.last().map(AsRef::as_ref).map_or_else(|| ROOT_MODULE_NAME, AsRef::as_ref)
    }

    /// Retrieve a child [`DamlModule`] by name or `None` if no such module exists.
    pub fn child_module<S: AsRef<str>>(&self, name: S) -> Option<&DamlModule<'_>> {
        self.child_modules.get(name.as_ref())
    }

    /// Retrieve a child [`DamlModule`] by name or `None` if no such module exists.
    pub fn child_module_path<'b, S: AsRef<str>>(&'a self, relative_path: &'b [S]) -> Option<&'a DamlModule<'a>> {
        match relative_path {
            [] => Some(self),
            [head, tail @ ..] => match self.child_module(head.as_ref()) {
                Some(m) => m.child_module_path(tail),
                None => None,
            },
        }
    }

    /// Retrieve a [`DamlData`] by name or `None` if no such data type exists.
    pub fn data_type<S: AsRef<str>>(&self, data: S) -> Option<&DamlData<'a>> {
        self.data_types.get(data.as_ref())
    }

    /// Retrieve or create a child [`DamlModule`] with `name`.
    #[doc(hidden)]
    pub(crate) fn child_module_or_new(&mut self, name: &'a str) -> &mut Self {
        let path = &self.path;
        self.child_modules.entry(Cow::from(name)).or_insert_with(|| {
            DamlModule::new_empty(path.iter().map(ToOwned::to_owned).chain(once(Cow::from(name))).collect())
        })
    }

    /// Populate this module with data taken from another module.
    ///
    /// Note that this does not copy the `child_modules` from the `other` [`DamlModule`] as this node may already have
    /// existing children.
    #[doc(hidden)]
    pub(crate) fn take_from(&mut self, other: Self) {
        debug_assert_eq!(self.path, other.path);
        self.flags = other.flags;
        self.data_types = other.data_types;
        self.exceptions = other.exceptions;
        self.synonyms = other.synonyms;
        #[cfg(feature = "full")]
        {
            self.values = other.values;
        }
    }

    /// Create an empty `DamlModule` with a given `path`.
    fn new_empty(path: Vec<Cow<'a, str>>) -> Self {
        Self {
            path,
            flags: DamlFeatureFlags::default(),
            synonyms: Vec::default(),
            child_modules: HashMap::default(),
            data_types: HashMap::default(),
            #[cfg(feature = "full")]
            values: Vec::default(),
            exceptions: Vec::default(),
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
        self.exceptions.iter().for_each(|ex| ex.accept(visitor));
        visitor.post_visit_module(self);
    }
}

impl ToStatic for DamlModule<'_> {
    type Static = DamlModule<'static>;

    fn to_static(&self) -> Self::Static {
        DamlModule {
            path: self.path.iter().map(ToStatic::to_static).collect(),
            flags: self.flags,
            synonyms: self.synonyms.iter().map(DamlDefTypeSyn::to_static).collect(),
            child_modules: self.child_modules.iter().map(|(k, v)| (k.to_static(), DamlModule::to_static(v))).collect(),
            data_types: self.data_types.iter().map(|(k, v)| (k.to_static(), DamlData::to_static(v))).collect(),
            #[cfg(feature = "full")]
            values: self.values.iter().map(DamlDefValue::to_static).collect(),
            exceptions: self.exceptions.iter().map(DamlDefException::to_static).collect(),
        }
    }
}

/// Definition of a DAML Type Synonym.
#[derive(Debug, Serialize, Clone)]
pub struct DamlDefTypeSyn<'a> {
    params: Vec<DamlTypeVarWithKind<'a>>,
    ty: DamlType<'a>,
    name: Vec<Cow<'a, str>>,
}

impl<'a> DamlDefTypeSyn<'a> {
    /// Create a type synonym.
    pub fn new(params: Vec<DamlTypeVarWithKind<'a>>, ty: DamlType<'a>, name: Vec<Cow<'a, str>>) -> Self {
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
    pub fn name(&self) -> impl Iterator<Item = &str> {
        self.name.iter().map(AsRef::as_ref)
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

impl ToStatic for DamlDefTypeSyn<'_> {
    type Static = DamlDefTypeSyn<'static>;

    fn to_static(&self) -> Self::Static {
        DamlDefTypeSyn::new(
            self.params.iter().map(DamlTypeVarWithKind::to_static).collect(),
            self.ty.to_static(),
            self.name.iter().map(ToStatic::to_static).collect(),
        )
    }
}

/// DOCME
#[derive(Debug, Serialize, Copy, Clone, Default)]
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
    pub fn forbid_party_literals(self) -> bool {
        self.forbid_party_literals
    }

    ///
    pub fn dont_divulge_contract_ids_in_create_arguments(self) -> bool {
        self.dont_divulge_contract_ids_in_create_arguments
    }

    ///
    pub fn dont_disclose_non_consuming_choices_to_observers(self) -> bool {
        self.dont_disclose_non_consuming_choices_to_observers
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlDefException<'a> {
    pub name: Vec<Cow<'a, str>>,
    #[cfg(feature = "full")]
    pub message: DamlExpr<'a>,
}

impl<'a> DamlDefException<'a> {
    pub const fn new(name: Vec<Cow<'a, str>>, #[cfg(feature = "full")] message: DamlExpr<'a>) -> Self {
        Self {
            name,
            #[cfg(feature = "full")]
            message,
        }
    }

    pub fn name(&self) -> impl Iterator<Item = &str> {
        self.name.iter().map(AsRef::as_ref)
    }

    #[cfg(feature = "full")]
    pub const fn message(&self) -> &DamlExpr<'a> {
        &self.message
    }
}

impl<'a> DamlVisitableElement<'a> for DamlDefException<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_def_exception(self);
        #[cfg(feature = "full")]
        self.message.accept(visitor);
        visitor.post_visit_def_exception(self);
    }
}

impl ToStatic for DamlDefException<'_> {
    type Static = DamlDefException<'static>;

    fn to_static(&self) -> Self::Static {
        DamlDefException::new(
            self.name.iter().map(ToStatic::to_static).collect(),
            #[cfg(feature = "full")]
            self.message.to_static(),
        )
    }
}
