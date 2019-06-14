use crate::data::value::{DamlRecord, DamlRecordField, DamlValue};
use crate::data::DamlIdentifier;

/// Helper for building a [`DamlRecord`].
#[derive(Debug, Default)]
pub struct DamlRecordBuilder {
    record_id: Option<DamlIdentifier>,
    fields: Vec<DamlRecordField>,
}

impl DamlRecordBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_id(mut self, record_id: DamlIdentifier) -> Self {
        self.record_id = Some(record_id);
        self
    }

    pub fn add_field(mut self, label: impl Into<String>, value: DamlValue) -> Self {
        self.fields.push(DamlRecordField::new(Some(label), value));
        self
    }

    pub fn build(self) -> DamlRecord {
        DamlRecord::new(self.fields, self.record_id)
    }
}
