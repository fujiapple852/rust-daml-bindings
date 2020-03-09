use daml_lf::protobuf_autogen::daml_lf_1::Module;

use crate::convert::archive::wrapper::payload::util::Required;
use crate::convert::archive::wrapper::payload::{DamlDataPayload, DamlTemplatePayload, InternableDottedName};
use crate::convert::error::{DamlCodeGenError, DamlCodeGenResult};
use std::collections::HashMap;
use std::convert::TryFrom;

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

impl<'a> TryFrom<&'a Module> for DamlModulePayload<'a> {
    type Error = DamlCodeGenError;

    fn try_from(module: &'a Module) -> DamlCodeGenResult<Self> {
        let path = InternableDottedName::from(module.name.as_ref().req()?);
        let templates = module
            .templates
            .iter()
            .flat_map(DamlTemplatePayload::try_from)
            .map(|t| Ok((t.name, t)))
            .collect::<DamlCodeGenResult<_>>()?;
        let data_types =
            module.data_types.iter().map(DamlDataPayload::try_from).collect::<DamlCodeGenResult<Vec<_>>>()?;
        Ok(Self {
            data_types,
            templates,
            path,
        })
    }
}

impl<'a> DamlModulePayload<'a> {
    /// Get a named template from this module.
    pub fn template(&self, name: &InternableDottedName<'a>) -> Option<&DamlTemplatePayload<'_>> {
        self.templates.get(name)
    }
}
