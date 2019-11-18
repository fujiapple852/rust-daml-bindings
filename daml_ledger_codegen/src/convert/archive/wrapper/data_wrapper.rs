use crate::convert::archive::wrapper::payload::{
    DamlChoicePayload, DamlDataPayload, DamlFieldPayload, DamlModulePayload, DamlPackagePayload, DamlTemplatePayload,
    EnumPayload, RecordPayload, VariantPayload,
};
use crate::convert::archive::wrapper::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamlDataWrapper<'a> {
    Record(DamlRecordWrapper<'a>),
    Template(DamlTemplateWrapper<'a>),
    Variant(DamlVariantWrapper<'a>),
    Enum(DamlEnumWrapper<'a>),
}

impl<'a> DamlDataWrapper<'a> {
    pub fn wrap(
        parent_archive: &'a DamlArchivePayload,
        parent_package: &'a DamlPackagePayload,
        parent_module: &'a DamlModulePayload,
        data: &'a DamlDataPayload,
    ) -> Self {
        match data {
            DamlDataPayload::Record(record) =>
                if let Some(template) = parent_module.template(record.name) {
                    DamlDataWrapper::Template(DamlTemplateWrapper::new(
                        parent_archive,
                        parent_package,
                        parent_module,
                        data,
                        template,
                    ))
                } else {
                    DamlDataWrapper::Record(DamlRecordWrapper::new(
                        parent_archive,
                        parent_package,
                        parent_module,
                        data,
                        record,
                    ))
                },
            DamlDataPayload::Variant(variant) => DamlDataWrapper::Variant(DamlVariantWrapper::new(
                parent_archive,
                parent_package,
                parent_module,
                data,
                variant,
            )),
            DamlDataPayload::Enum(data_enum) => DamlDataWrapper::Enum(DamlEnumWrapper::new(
                parent_archive,
                parent_package,
                parent_module,
                //                data,
                data_enum,
            )),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DamlTemplateWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub parent_package: &'a DamlPackagePayload<'a>,
    pub parent_module: &'a DamlModulePayload<'a>,
    pub parent_data: &'a DamlDataPayload<'a>,
    pub payload: &'a DamlTemplatePayload<'a>,
}

impl<'a> PartialEq for DamlTemplateWrapper<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.payload.name == other.payload.name
    }
}

impl<'a> DamlTemplateWrapper<'a> {
    pub fn new(
        parent_archive: &'a DamlArchivePayload<'a>,
        parent_package: &'a DamlPackagePayload<'a>,
        parent_module: &'a DamlModulePayload<'a>,
        parent_data: &'a DamlDataPayload<'a>,
        payload: &'a DamlTemplatePayload<'a>,
    ) -> Self {
        Self {
            parent_archive,
            parent_package,
            parent_module,
            parent_data,
            payload,
        }
    }

    pub fn choices(self) -> impl Iterator<Item = DamlChoiceWrapper<'a>> {
        self.payload.choices.iter().map(move |choice| self.choice(choice))
    }

    fn choice(self, choice: &'a DamlChoicePayload) -> DamlChoiceWrapper<'a> {
        DamlChoiceWrapper {
            parent_archive: self.parent_archive,
            parent_package: self.parent_package,
            parent_module: self.parent_module,
            parent_data: &self.parent_data,
            payload: choice,
        }
    }

    pub fn fields(self) -> impl Iterator<Item = DamlFieldWrapper<'a>> {
        match self.parent_data {
            DamlDataPayload::Record(record) => record.fields.iter().map(move |field| self.wrap_field(field)),
            _ => panic!("expected parent to be Record"),
        }
    }

    fn wrap_field(self, field: &'a DamlFieldPayload) -> DamlFieldWrapper<'a> {
        DamlFieldWrapper::wrap(self.parent_archive, self.parent_package, self.parent_module, self.parent_data, field)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DamlChoiceWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub parent_package: &'a DamlPackagePayload<'a>,
    pub parent_module: &'a DamlModulePayload<'a>,
    pub parent_data: &'a DamlDataPayload<'a>,
    pub payload: &'a DamlChoicePayload<'a>,
}

impl<'a> DamlChoiceWrapper<'a> {
    pub fn argument_type(self) -> DamlTypeWrapper<'a> {
        DamlTypeWrapper::wrap(
            self.parent_archive,
            self.parent_package,
            self.parent_module,
            self.parent_data,
            &self.payload.argument_type,
        )
    }

    pub fn return_type(self) -> DamlTypeWrapper<'a> {
        DamlTypeWrapper::wrap(
            self.parent_archive,
            self.parent_package,
            self.parent_module,
            self.parent_data,
            &self.payload.return_type,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DamlRecordWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub parent_package: &'a DamlPackagePayload<'a>,
    pub parent_module: &'a DamlModulePayload<'a>,
    pub parent_data: &'a DamlDataPayload<'a>,
    pub payload: &'a RecordPayload<'a>,
}

impl<'a> PartialEq for DamlRecordWrapper<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.payload.name == other.payload.name
    }
}

impl<'a> DamlRecordWrapper<'a> {
    pub fn new(
        parent_archive: &'a DamlArchivePayload<'a>,
        parent_package: &'a DamlPackagePayload<'a>,
        parent_module: &'a DamlModulePayload<'a>,
        parent_data: &'a DamlDataPayload<'a>,
        payload: &'a RecordPayload<'a>,
    ) -> Self {
        Self {
            parent_archive,
            parent_package,
            parent_module,
            parent_data,
            payload,
        }
    }

    pub fn fields(self) -> impl Iterator<Item = DamlFieldWrapper<'a>> {
        self.payload.fields.iter().map(move |field| self.wrap_field(field))
    }

    fn wrap_field(self, field: &'a DamlFieldPayload) -> DamlFieldWrapper<'a> {
        DamlFieldWrapper::wrap(self.parent_archive, self.parent_package, self.parent_module, self.parent_data, field)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DamlVariantWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub parent_package: &'a DamlPackagePayload<'a>,
    pub parent_module: &'a DamlModulePayload<'a>,
    pub parent_data: &'a DamlDataPayload<'a>,
    pub payload: &'a VariantPayload<'a>,
}

impl<'a> PartialEq for DamlVariantWrapper<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.payload.name == other.payload.name
    }
}

impl<'a> DamlVariantWrapper<'a> {
    pub fn new(
        parent_archive: &'a DamlArchivePayload<'a>,
        parent_package: &'a DamlPackagePayload<'a>,
        parent_module: &'a DamlModulePayload<'a>,
        parent_data: &'a DamlDataPayload<'a>,
        payload: &'a VariantPayload<'a>,
    ) -> Self {
        Self {
            parent_archive,
            parent_package,
            parent_module,
            parent_data,
            payload,
        }
    }

    pub fn fields(self) -> impl Iterator<Item = DamlFieldWrapper<'a>> {
        self.payload.fields.iter().map(move |field| self.wrap_field(field))
    }

    fn wrap_field(self, field: &'a DamlFieldPayload) -> DamlFieldWrapper<'a> {
        DamlFieldWrapper::wrap(self.parent_archive, self.parent_package, self.parent_module, self.parent_data, field)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DamlEnumWrapper<'a> {
    pub parent_archive: &'a DamlArchivePayload<'a>,
    pub parent_package: &'a DamlPackagePayload<'a>,
    pub parent_module: &'a DamlModulePayload<'a>,
    pub payload: &'a EnumPayload<'a>,
}

impl<'a> PartialEq for DamlEnumWrapper<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.payload.name == other.payload.name
    }
}

impl<'a> DamlEnumWrapper<'a> {
    pub fn new(
        parent_archive: &'a DamlArchivePayload<'a>,
        parent_package: &'a DamlPackagePayload<'a>,
        parent_module: &'a DamlModulePayload<'a>,
        payload: &'a EnumPayload<'a>,
    ) -> Self {
        Self {
            parent_archive,
            parent_package,
            parent_module,
            payload,
        }
    }
}
