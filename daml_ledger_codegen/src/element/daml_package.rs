use crate::element::daml_module::DamlModule;

#[derive(Debug)]
pub struct DamlPackage<'a> {
    pub name: &'a str,
    pub version: Option<&'a str>,
    pub package_id: &'a str,
    pub root_module: DamlModule<'a>,
}

impl<'a> DamlPackage<'a> {
    pub fn new(name: &'a str, package_id: &'a str, version: Option<&'a str>, root_module: DamlModule<'a>) -> Self {
        Self {
            name,
            package_id,
            version,
            root_module,
        }
    }
}
