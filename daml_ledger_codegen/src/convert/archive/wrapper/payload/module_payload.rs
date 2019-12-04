use daml_lf::protobuf_autogen::daml_lf_1::Module;

use crate::convert::archive::wrapper::payload::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct DamlModulePayload<'a> {
    pub data_types: Vec<DamlDataPayload<'a>>,
    pub templates: HashMap<InternableDottedName<'a>, DamlTemplatePayload<'a>>,
    pub path: InternableDottedName<'a>,
}

impl<'a> PartialEq for DamlModulePayload<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<'a> From<&'a Module> for DamlModulePayload<'a> {
    fn from(module: &'a Module) -> Self {
        let path = InternableDottedName::from(module.name.as_ref().expect("Module.name"));
        let templates = module.templates.iter().map(DamlTemplatePayload::from).map(|t| (t.name, t)).collect();
        let data_types = module.data_types.iter().map(DamlDataPayload::from).collect::<Vec<_>>();
        Self {
            data_types,
            templates,
            path,
        }
    }
}

impl<'a> DamlModulePayload<'a> {
    /// Get a named template from this module.
    pub fn template(&self, name: &InternableDottedName<'a>) -> Option<&DamlTemplatePayload> {
        self.templates.get(name)
    }
}
