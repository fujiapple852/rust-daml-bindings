use crate::element::daml_module::DamlModule;
use crate::element::visitor::DamlElementVisitor;
use crate::element::DamlVisitableElement;
use crate::LanguageVersion;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct DamlPackage<'a> {
    name: &'a str,
    package_id: &'a str,
    version: Option<&'a str>,
    language_version: LanguageVersion,
    root_module: DamlModule<'a>,
}

impl<'a> DamlPackage<'a> {
    pub const fn new(
        name: &'a str,
        package_id: &'a str,
        version: Option<&'a str>,
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

    pub const fn name(&self) -> &str {
        self.name
    }

    pub const fn package_id(&self) -> &str {
        self.package_id
    }

    pub const fn version(&self) -> Option<&'a str> {
        self.version
    }

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
