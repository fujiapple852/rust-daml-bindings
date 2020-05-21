use crate::data::value::DamlValue;
use crate::data::DamlError;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{RecordField, Value};
use crate::util::Required;
use std::convert::TryFrom;

/// A representation of a single field on a DAML record.
#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd)]
pub struct DamlRecordField {
    label: Option<String>,
    value: DamlValue,
}

impl DamlRecordField {
    pub fn new(label: Option<impl Into<String>>, value: impl Into<DamlValue>) -> Self {
        Self {
            label: label.map(Into::into),
            value: value.into(),
        }
    }

    pub const fn label(&self) -> &Option<String> {
        &self.label
    }

    pub const fn value(&self) -> &DamlValue {
        &self.value
    }
}

impl TryFrom<RecordField> for DamlRecordField {
    type Error = DamlError;

    fn try_from(field: RecordField) -> Result<Self, Self::Error> {
        let label = field.label;
        let value: DamlValue = field.value.req().and_then(DamlValue::try_from)?;
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
        Self {
            label: daml_record_field.label.unwrap_or_default(),
            value: Some(Value::from(daml_record_field.value)),
        }
    }
}
