use crate::data::value::DamlValue;
use crate::data::{DamlError, DamlIdentifier};
use crate::grpc_protobuf::com::daml::ledger::api::v1::{Enum, Identifier, Value, Variant};
use crate::util::Required;
use std::convert::TryFrom;

/// A representation of a DAML variant field.
#[derive(Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd)]
pub struct DamlVariant {
    pub variant_id: Option<DamlIdentifier>,
    pub constructor: String,
    pub value: Box<DamlValue>,
}

impl DamlVariant {
    pub fn new(constructor: impl Into<String>, value: Box<DamlValue>, variant_id: Option<DamlIdentifier>) -> Self {
        Self {
            variant_id,
            constructor: constructor.into(),
            value,
        }
    }

    pub const fn variant_id(&self) -> &Option<DamlIdentifier> {
        &self.variant_id
    }

    pub fn constructor(&self) -> &str {
        &self.constructor
    }

    pub const fn value(&self) -> &DamlValue {
        &self.value
    }

    pub fn take_value(self) -> Box<DamlValue> {
        self.value
    }
}

impl TryFrom<Variant> for DamlVariant {
    type Error = DamlError;

    fn try_from(v: Variant) -> Result<Self, Self::Error> {
        Ok(Self::new(
            v.constructor,
            Box::new(v.value.req().and_then(|q| DamlValue::try_from(*q))?),
            v.variant_id.map(DamlIdentifier::from),
        ))
    }
}

impl From<DamlVariant> for Variant {
    fn from(daml_variant: DamlVariant) -> Self {
        Self {
            constructor: daml_variant.constructor,
            value: Some(Box::new(Value::from(*daml_variant.value))),
            variant_id: None,
        }
    }
}

/// A representation of a DAML enum, a value with finite set of alternative representations.
#[derive(Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd)]
pub struct DamlEnum {
    enum_id: Option<DamlIdentifier>,
    constructor: String,
}

impl DamlEnum {
    pub fn new(constructor: impl Into<String>, enum_id: Option<DamlIdentifier>) -> Self {
        Self {
            enum_id,
            constructor: constructor.into(),
        }
    }

    /// Omitted from the transaction stream when verbose streaming is not enabled.
    ///
    /// Optional when submitting commands.
    pub const fn enum_id(&self) -> &Option<DamlIdentifier> {
        &self.enum_id
    }

    /// Determines which of the Variant's alternatives is encoded in this message.
    ///
    /// Must be a valid `NameString`.
    pub fn constructor(&self) -> &str {
        &self.constructor
    }
}

impl From<Enum> for DamlEnum {
    fn from(e: Enum) -> Self {
        Self::new(e.constructor, e.enum_id.map(DamlIdentifier::from))
    }
}

impl From<DamlEnum> for Enum {
    fn from(daml_enum: DamlEnum) -> Self {
        Self {
            enum_id: daml_enum.enum_id.map(Identifier::from),
            constructor: daml_enum.constructor,
        }
    }
}
