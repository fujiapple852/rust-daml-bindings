use crate::element::daml_module::DamlModule;
use crate::element::visitor::DamlElementVisitor;
use crate::element::DamlVisitableElement;
use crate::LanguageVersion;
use bounded_static::ToStatic;
use serde::Serialize;
use std::borrow::Cow;

/// A Daml package.
#[derive(Debug, Serialize, Clone, ToStatic)]
pub struct DamlPackage<'a> {
    name: Cow<'a, str>,
    package_id: Cow<'a, str>,
    version: Option<Cow<'a, str>>,
    language_version: LanguageVersion,
    root_module: DamlModule<'a>,
}

impl<'a> DamlPackage<'a> {
    pub const fn new(
        name: Cow<'a, str>,
        package_id: Cow<'a, str>,
        version: Option<Cow<'a, str>>,
        language_version: LanguageVersion,
        root_module: DamlModule<'a>,
    ) -> Self {
        Self {
            name,
            package_id,
            version,
            language_version,
            root_module,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    /// Package version.
    pub fn version(&self) -> Option<&str> {
        self.version.as_ref().map(AsRef::as_ref)
    }

    /// The Daml LF language version.
    pub const fn language_version(&self) -> LanguageVersion {
        self.language_version
    }

    pub const fn root_module(&self) -> &DamlModule<'a> {
        &self.root_module
    }
}

impl<'a> DamlVisitableElement<'a> for DamlPackage<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_package(self);
        self.root_module.accept(visitor);
        visitor.post_visit_package(self);
    }
}
