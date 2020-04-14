use crate::convert::archive_payload::DamlArchivePayload;
use crate::convert::data_payload::{DamlDataPayload, DamlTemplatePayload};
use crate::convert::interned::InternableDottedName;
use crate::convert::package_payload::DamlPackagePayload;
use crate::convert::util::Required;
use crate::convert::wrapper::DamlPayloadDataWrapper;
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::protobuf_autogen::daml_lf_1::Module;
use std::collections::HashMap;
use std::convert::TryFrom;

///
#[derive(Debug, Clone, Copy)]
pub struct DamlModuleWrapper<'a> {
    pub archive: &'a DamlArchivePayload<'a>,
    pub package: &'a DamlPackagePayload<'a>,
    pub module: &'a DamlModulePayload<'a>,
}

impl<'a> DamlModuleWrapper<'a> {
    pub fn with_data(self, data: &'a DamlDataPayload<'_>) -> DamlPayloadDataWrapper<'a> {
        DamlPayloadDataWrapper {
            archive: self.archive,
            package: self.package,
            module: self.module,
            data,
        }
    }
}

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
    type Error = DamlLfConvertError;

    fn try_from(module: &'a Module) -> DamlLfConvertResult<Self> {
        let path = InternableDottedName::from(module.name.as_ref().req()?);
        let templates = module
            .templates
            .iter()
            .flat_map(DamlTemplatePayload::try_from)
            .map(|t| Ok((t.name, t)))
            .collect::<DamlLfConvertResult<_>>()?;
        let data_types =
            module.data_types.iter().map(DamlDataPayload::try_from).collect::<DamlLfConvertResult<Vec<_>>>()?;
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
