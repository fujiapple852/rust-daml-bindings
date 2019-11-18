use crate::data::value::DamlValue;
use crate::data::DamlError;
use crate::grpc_protobuf_autogen::value::RecordField;
use std::convert::{TryFrom, TryInto};

/// A representation of a single field on a DAML record.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlRecordField {
    pub label: Option<String>,
    pub value: DamlValue,
}

impl DamlRecordField {
    pub fn new(label: Option<impl Into<String>>, value: impl Into<DamlValue>) -> Self {
        Self {
            label: label.map(Into::into),
            value: value.into(),
        }
    }

    pub fn label(&self) -> &Option<String> {
        &self.label
    }

    pub fn value(&self) -> &DamlValue {
        &self.value
    }
}

impl TryFrom<RecordField> for DamlRecordField {
    type Error = DamlError;

    fn try_from(mut field: RecordField) -> Result<Self, Self::Error> {
        let label = field.take_label();
        let value: DamlValue = field.take_value().try_into()?;
        Ok(Self::new(
            if label.is_empty() {
                None
            } else {
                Some(label)
            },
            value,
        ))
    }
}

impl From<DamlRecordField> for RecordField {
    fn from(daml_record_field: DamlRecordField) -> Self {
        let mut field = Self::new();
        if let Some(l) = daml_record_field.label {
            field.set_label(l);
        }
        field.set_value(daml_record_field.value.into());
        field
    }
}
