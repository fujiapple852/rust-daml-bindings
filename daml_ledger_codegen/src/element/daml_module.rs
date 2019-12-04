use std::collections::HashMap;

use crate::element::daml_data::DamlData;

#[derive(Debug)]
pub struct DamlModule<'a> {
    pub path: Vec<&'a str>,
    pub child_modules: HashMap<&'a str, DamlModule<'a>>,
    pub data_types: HashMap<&'a str, DamlData<'a>>,
}

impl<'a> DamlModule<'a> {
    pub fn new(path: Vec<&'a str>) -> Self {
        Self {
            path,
            child_modules: HashMap::default(),
            data_types: HashMap::default(),
        }
    }

    pub fn new_root() -> Self {
        Self::new(vec![])
    }

    pub fn name(&self) -> &str {
        self.path.last().unwrap_or_else(|| &"root")
    }
}
