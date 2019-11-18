use std::collections::HashMap;

use daml_lf::protobuf_autogen::daml_lf_1::Module;

use crate::convert::archive::wrapper::payload::*;

#[derive(Debug)]
pub struct DamlModulePayload<'a> {
    pub data_types: HashMap<&'a str, DamlDataPayload<'a>>,
    pub templates: HashMap<&'a str, DamlTemplatePayload<'a>>,
    pub path: &'a [String],
    pub children: HashMap<&'a str, DamlModulePayload<'a>>,
}

impl<'a> PartialEq for DamlModulePayload<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<'a> DamlModulePayload<'a> {
    /// The name of the module.
    ///
    /// The module name is the last element of the module path.
    pub fn name(&self) -> &str {
        self.path.last().expect("Module does not have a name")
    }

    /// Get a named data type from this module.
    pub fn data_type(&self, name: &str) -> Option<&DamlDataPayload> {
        self.data_types.get(name)
    }

    /// Get a named template from this module.
    pub fn template(&self, name: &str) -> Option<&DamlTemplatePayload> {
        self.templates.get(name)
    }

    /// Build a tree of modules `DamlModulePayload` from a list of `Module`
    ///
    /// Recursively builds a tree of modules from the flat list provided and return the root module.
    pub fn from_modules(modules: &'a [Module]) -> Self {
        let mut root = Self::new(&[]);
        for module in modules {
            let path = module.name.as_ref().expect("Module.name").segments.as_slice();
            let templates = module.templates.iter().map(DamlTemplatePayload::from).map(|t| (t.name, t)).collect();
            let data_types = module
                .data_types
                .iter()
                .map(|dt| (leaf_name(dt.name.as_ref().expect("DefDataType.name")), DamlDataPayload::from(dt)))
                .collect();
            Self::add_module_to_tree(&mut root, &path, &path, data_types, templates);
        }
        root
    }

    /// Get this or a sub-module for a given path.
    pub fn module_by_path(&self, path: &'a [String]) -> Option<&Self> {
        if path == self.path {
            Some(self)
        } else {
            self.children.values().find_map(|m| m.module_by_path(path))
        }
    }

    fn add_module_to_tree(
        node: &mut Self,
        full_path: &'a [String],
        remaining_path: &'a [String],
        data_types: HashMap<&'a str, DamlDataPayload<'a>>,
        templates: HashMap<&'a str, DamlTemplatePayload<'a>>,
    ) {
        if let Some(child_mod_name) = remaining_path.first() {
            let child_mod_path = &full_path[..=full_path.len() - remaining_path.len()];
            let entry = node.children.entry(child_mod_name).or_insert_with(|| Self::new(child_mod_path));
            Self::add_module_to_tree(entry, full_path, &remaining_path[1..], data_types, templates)
        } else {
            node.data_types = data_types;
            node.templates = templates;
        }
    }

    fn new(path: &'a [String]) -> Self {
        Self {
            data_types: HashMap::default(),
            templates: HashMap::default(),
            path,
            children: HashMap::default(),
        }
    }
}
