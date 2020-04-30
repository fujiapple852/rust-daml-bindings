use crate::element::daml_module::DamlModule;
use crate::element::visitor::DamlElementVisitor;
use crate::element::DamlVisitableElement;
use crate::LanguageVersion;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DamlPackage<'a> {
    pub name: &'a str,
    pub package_id: &'a str,
    pub version: Option<&'a str>,
    pub language_version: LanguageVersion,
    pub root_module: DamlModule<'a>,
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
}

impl<'a> DamlVisitableElement<'a> for DamlPackage<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_package(self);
        self.root_module.accept(visitor);
        visitor.post_visit_package(self);
    }
}
