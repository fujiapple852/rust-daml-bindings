use crate::data::identifier::DamlIdentifier;
use crate::data::value::DamlValue;
use crate::data::DamlError;
use crate::grpc_protobuf_autogen::value::Variant;
use crate::grpc_protobuf_autogen::value::{Enum, Value};
use std::convert::{TryFrom, TryInto};

/// A representation of a DAML variant field.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlVariant {
    variant_id: Option<DamlIdentifier>,
    constructor: String,
    value: Box<DamlValue>,
}

impl DamlVariant {
    pub fn new(constructor: impl Into<String>, value: Box<DamlValue>, variant_id: Option<DamlIdentifier>) -> Self {
        Self {
            variant_id,
            constructor: constructor.into(),
            value,
        }
    }

    pub fn variant_id(&self) -> &Option<DamlIdentifier> {
        &self.variant_id
    }

    pub fn constructor(&self) -> &str {
        &self.constructor
    }

    pub fn value(&self) -> &DamlValue {
        &self.value
    }

    pub fn take_value(self) -> Box<DamlValue> {
        self.value
    }
}

impl TryFrom<Variant> for DamlVariant {
    type Error = DamlError;

    fn try_from(mut v: Variant) -> Result<Self, Self::Error> {
        let value = v.take_value().try_into()?;
        Ok(Self::new(
            v.take_constructor(),
            Box::new(value),
            if v.has_variant_id() {
                Some(v.take_variant_id().into())
            } else {
                None
            },
        ))
    }
}

impl From<DamlVariant> for Variant {
    fn from(daml_variant: DamlVariant) -> Self {
        let mut variant = Self::new();
        if let Some(id) = daml_variant.variant_id {
            variant.set_variant_id(id.into());
        }
        variant.set_constructor(daml_variant.constructor);
        variant.set_value(Into::<Value>::into(*(daml_variant.value)));
        variant
    }
}

/// A representation of a DAML enum, a value with finite set of alternative representations.
#[derive(Debug, Eq, PartialEq, Clone)]
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
    pub fn enum_id(&self) -> &Option<DamlIdentifier> {
        &self.enum_id
    }

    /// Determines which of the Variant's alternatives is encoded in this message.
    ///
    /// Must be a valid NameString.
    pub fn constructor(&self) -> &str {
        &self.constructor
    }
}

impl From<Enum> for DamlEnum {
    fn from(mut e: Enum) -> Self {
        Self::new(
            e.take_constructor(),
            if e.has_enum_id() {
                Some(e.take_enum_id().into())
            } else {
                None
            },
        )
    }
}

impl From<DamlEnum> for Enum {
    fn from(daml_enum: DamlEnum) -> Self {
        let mut grpc_enum = Self::new();
        if let Some(id) = daml_enum.enum_id {
            grpc_enum.set_enum_id(id.into());
        }
        grpc_enum.set_constructor(daml_enum.constructor);
        grpc_enum
    }
}
