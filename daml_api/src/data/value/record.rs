use crate::data::value::{DamlRecordField, DamlValue};
use crate::data::{DamlError, DamlIdentifier, DamlResult};
use crate::grpc_protobuf::com::daml::ledger::api::v1::{Identifier, Record, RecordField};
use std::convert::{TryFrom, TryInto};

/// A representation of the fields on a DAML `template` or `data` construct.
#[derive(Debug, PartialEq, Eq, Default, Clone, Hash, Ord, PartialOrd)]
pub struct DamlRecord {
    pub record_id: Option<DamlIdentifier>,
    pub fields: Vec<DamlRecordField>,
}

impl DamlRecord {
    pub fn empty() -> Self {
        Self {
            record_id: None,
            fields: vec![],
        }
    }

    pub fn new(fields: impl Into<Vec<DamlRecordField>>, record_id: Option<impl Into<DamlIdentifier>>) -> Self {
        Self {
            record_id: record_id.map(Into::into),
            fields: fields.into(),
        }
    }

    pub fn record_id(&self) -> &Option<DamlIdentifier> {
        &self.record_id
    }

    pub fn fields(&self) -> &Vec<DamlRecordField> {
        &self.fields
    }

    pub fn field(&self, label: &str) -> DamlResult<&DamlValue> {
        self.fields
            .iter()
            .find_map(|rec| match rec.label() {
                Some(ll) if ll == label => Some(rec.value()),
                _ => None,
            })
            .ok_or_else(|| DamlError::UnknownField(label.to_owned()))
    }

    /// Apply a DAML data extractor function.
    ///
    /// See [`DamlValue::extract`] for details an examples.
    pub fn extract<'a, R, F>(&'a self, f: F) -> DamlResult<R>
    where
        F: Fn(&'a Self) -> DamlResult<R>,
    {
        f(self)
    }
}

impl TryFrom<Record> for DamlRecord {
    type Error = DamlError;

    fn try_from(record: Record) -> Result<Self, Self::Error> {
        let fields = record.fields.into_iter().map(TryInto::try_into).collect::<DamlResult<Vec<DamlRecordField>>>()?;
        Ok(Self::new(fields, record.record_id.map(DamlIdentifier::from)))
    }
}

impl From<DamlRecord> for Record {
    fn from(daml_record: DamlRecord) -> Self {
        Self {
            record_id: daml_record.record_id.map(Identifier::from),
            fields: daml_record.fields.into_iter().map(RecordField::from).collect(),
        }
    }
}

impl TryFrom<DamlValue> for DamlRecord {
    type Error = DamlError;

    fn try_from(value: DamlValue) -> Result<Self, Self::Error> {
        value.try_take_record()
    }
}
