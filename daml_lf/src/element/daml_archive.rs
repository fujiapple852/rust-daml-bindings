use std::collections::HashMap;

use crate::element::daml_package::DamlPackage;
use crate::element::visitor::{DamlElementVisitor, DamlVisitableElement};
use crate::element::{serialize, DamlData, DamlDataRef};
use itertools::Itertools;
use serde::Serialize;

#[derive(Debug, Serialize, Clone, Default)]
pub struct DamlArchive<'a> {
    name: &'a str,
    #[serde(serialize_with = "serialize::serialize_map")]
    packages: HashMap<&'a str, DamlPackage<'a>>,
}

impl<'a> DamlArchive<'a> {
    ///
    pub const fn new(name: &'a str, packages: HashMap<&'a str, DamlPackage<'a>>) -> Self {
        Self {
            name,
            packages,
        }
    }

    pub const fn name(&self) -> &str {
        self.name
    }

    pub const fn packages(&self) -> &HashMap<&'a str, DamlPackage<'a>> {
        &self.packages
    }

    /// Retrieve a `DamlData` contained within this `DamlArchive` referred to by the supplied `DamlDataRef` or `None` if
    /// not such data item exists.
    ///
    /// TODO document this
    pub fn data_by_ref<'b>(&'a self, data_ref: &'b DamlDataRef<'_>) -> Option<&'a DamlData<'a>> {
        let (package_name, module_path, data_name) = data_ref.reference_parts();
        self.packages.get(package_name)?.root_module().child_module_path(module_path)?.data_types().get(data_name)
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
