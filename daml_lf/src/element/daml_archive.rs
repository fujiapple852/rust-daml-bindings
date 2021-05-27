use crate::element::daml_package::DamlPackage;
use crate::element::visitor::{DamlElementVisitor, DamlVisitableElement};
use crate::element::{serialize, DamlData, DamlTyCon};
#[cfg(feature = "full")]
use crate::element::{DamlDefValue, DamlValueName};
use crate::owned::ToStatic;
use itertools::Itertools;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone, Default)]
pub struct DamlArchive<'a> {
    name: Cow<'a, str>,
    main_package_id: Cow<'a, str>,
    #[serde(serialize_with = "serialize::serialize_map")]
    packages: HashMap<Cow<'a, str>, DamlPackage<'a>>,
}

impl<'a> DamlArchive<'a> {
    ///
    pub const fn new(
        name: Cow<'a, str>,
        main_package_id: Cow<'a, str>,
        packages: HashMap<Cow<'a, str>, DamlPackage<'a>>,
    ) -> Self {
        Self {
            name,
            main_package_id,
            packages,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the package id of the main `DamlPackage` contained in this `DamlArchive`.
    pub fn main_package_id(&self) -> &str {
        &self.main_package_id
    }

    /// Return an Iterator of the [`DamlPackage`] in this [`DamlArchive`].
    pub fn packages(&self) -> impl Iterator<Item = &DamlPackage<'_>> {
        self.packages.values()
    }

    /// Return the first [`DamlPackage`] in this [`DamlArchive`] which has the given `name` or `None` if no such
    /// package exists.
    pub fn package_by_name(&self, name: &str) -> Option<&DamlPackage<'_>> {
        self.packages.values().find(|p| p.name() == name)
    }

    /// Return the main [`DamlPackage`] in this [`DamlArchive`] or `None` if no such package exists.`
    pub fn main_package(&self) -> Option<&DamlPackage<'_>> {
        self.packages.get(&self.main_package_id)
    }

    /// Retrieve a `DamlData` contained within this `DamlArchive` referred to by the supplied `DamlTyCon` or `None` if
    /// not such data item exists.
    ///
    /// DOCME
    pub fn data_by_tycon<'b>(&'a self, tycon: &'b DamlTyCon<'_>) -> Option<&'a DamlData<'a>> {
        let (package_id, module_path, data_name) = tycon.tycon().reference_parts();
        self.data(package_id, module_path, data_name)
    }

    /// Retrieve a `DamlData` contained within this `DamlArchive` referred to by the supplied package id, module path &
    /// name or `None` if not such data item exists.
    ///
    /// DOCME
    pub fn data<P, M, D>(&'a self, package_id: P, module_path: &[M], data_name: D) -> Option<&'a DamlData<'a>>
    where
        P: AsRef<str>,
        M: AsRef<str>,
        D: AsRef<str>,
    {
        self.packages
            .get(package_id.as_ref())?
            .root_module()
            .child_module_path(module_path)?
            .data_type(data_name.as_ref())
    }

    /// Retrieve a `DamlDefValue` for a given `DamlValueName` or `None` if no such value exists in this `DamlArchive`.
    ///
    /// DOCME
    #[cfg(feature = "full")]
    pub fn value_by_name<'b>(&'a self, name: &'b DamlValueName<'_>) -> Option<&'a DamlDefValue<'a>> {
        let (package_id, module_path, name) = name.reference_parts();
        self.value(package_id, module_path, name)
    }

    /// Retrieve a `DamlDefValue` for the supplied package id, module path & name or `None` if no such value exists in
    /// this `DamlArchive`.
    ///
    /// DOCME
    #[cfg(feature = "full")]
    pub fn value<P, M, D>(&'a self, package_id: P, module_path: &[M], name: D) -> Option<&'a DamlDefValue<'a>>
    where
        P: AsRef<str>,
        M: AsRef<str>,
        D: AsRef<str>,
    {
        self.packages.get(package_id.as_ref())?.root_module().child_module_path(module_path)?.value(name.as_ref())
    }
}

impl<'a> DamlVisitableElement<'a> for DamlArchive<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_archive(self);
        if visitor.sort_elements() {
            self.packages.values().sorted_by_key(|p| p.name()).for_each(|package| package.accept(visitor));
        } else {
            self.packages.values().for_each(|package| package.accept(visitor));
        }
        visitor.post_visit_archive(self);
    }
}

impl ToStatic for DamlArchive<'_> {
    type Static = DamlArchive<'static>;

    fn to_static(&self) -> Self::Static {
        DamlArchive::new(
            self.name.to_static(),
            self.main_package_id.to_static(),
            self.packages.iter().map(|(k, v)| (k.to_static(), DamlPackage::to_static(v))).collect(),
        )
    }
}
