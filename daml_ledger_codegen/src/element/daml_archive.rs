use std::collections::HashMap;

use crate::element::daml_package::DamlPackage;

#[derive(Debug)]
pub struct DamlArchive<'a> {
    pub name: &'a str,
    pub packages: HashMap<&'a str, DamlPackage<'a>>,
}

impl<'a> DamlArchive<'a> {
    pub fn new(name: &'a str, packages: HashMap<&'a str, DamlPackage<'a>>) -> Self {
        Self {
            name,
            packages,
        }
    }
}
