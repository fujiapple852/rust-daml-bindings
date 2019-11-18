use std::collections::HashMap;

use crate::element::daml_data::DamlData;

#[derive(Debug)]
pub struct DamlModule<'a> {
    pub name: &'a str,
    pub path: &'a [String],
    pub child_modules: HashMap<&'a str, DamlModule<'a>>,
    pub data_types: HashMap<&'a str, DamlData<'a>>,
}

impl<'a> DamlModule<'a> {
    pub fn new(
        path: &'a [String],
        child_modules: HashMap<&'a str, DamlModule<'a>>,
        data_types: Vec<DamlData<'a>>,
    ) -> Self {
        Self {
            name: path.last().map_or_else(|| "root", String::as_str),
            path,
            child_modules,
            data_types: data_types.into_iter().map(|dt| (dt.name(), dt)).collect(),
        }
    }
}
