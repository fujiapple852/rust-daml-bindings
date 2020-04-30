use crate::data::value::DamlValue;
use crate::data::DamlResult;
use crate::primitive_types::{
    DamlBool, DamlContractId, DamlDate, DamlInt64, DamlNumeric, DamlParty, DamlText, DamlTimestamp, DamlUnit,
};
use std::collections::HashMap;

/// Marker trait for types which can be serialized to a [`DamlValue`].
pub trait DamlSerializableType: Sized {}

impl DamlSerializableType for DamlUnit {}
impl DamlSerializableType for DamlBool {}
impl DamlSerializableType for DamlInt64 {}
impl DamlSerializableType for DamlText {}
impl DamlSerializableType for DamlParty {}
impl DamlSerializableType for DamlContractId {}
impl DamlSerializableType for DamlNumeric {}
impl DamlSerializableType for DamlTimestamp {}
impl DamlSerializableType for DamlDate {}
impl<T> DamlSerializableType for Box<T> where T: DamlSerializeInto<DamlValue> + DamlSerializableType {}
impl<T> DamlSerializableType for Option<T> where T: DamlSerializeInto<DamlValue> + DamlSerializableType {}
impl<T> DamlSerializableType for Vec<T> where T: DamlSerializeInto<DamlValue> + DamlSerializableType {}
#[allow(clippy::implicit_hasher)]
impl<T> DamlSerializableType for HashMap<String, T> where T: DamlSerializeInto<DamlValue> + DamlSerializableType {}

/// Serialize from a concrete [`DamlValueType`] to a [`DamlValue`].
pub trait DamlSerializeFrom<T>: Sized
where
    T: DamlSerializableType,
{
    fn serialize_from(_: T) -> Self;
}

/// Serialize a concrete [`DamlValueType`] type into a [`DamlValue`].
pub trait DamlSerializeInto<T = DamlValue>: DamlSerializableType {
    fn serialize_into(self) -> T;
}

/// Blanket impl for all concrete `DamlValueType` types which implement [`DamlSerializeFrom`].
impl<T, U> DamlSerializeInto<U> for T
where
    T: DamlSerializableType,
    U: DamlSerializeFrom<T>,
{
    fn serialize_into(self) -> U {
        U::serialize_from(self)
    }
}

/// Deserialize from a [`DamlValue`] to a concrete [`DamlValueType`] type.
pub trait DamlDeserializeFrom: DamlDeserializableType {
    fn deserialize_from(value: DamlValue) -> DamlResult<Self>;
}

/// Deserialize a [`DamlValue`] into a concrete [`DamlValueType`] type.
pub trait DamlDeserializeInto<T: DamlDeserializableType> {
    fn deserialize_into(self) -> DamlResult<T>;
}

/// Blanket [`DamlDeserializeInto`] impl for all types which implement [`DamlDeserialize`].
impl<T> DamlDeserializeInto<T> for DamlValue
where
    T: DamlDeserializeFrom,
{
    fn deserialize_into(self) -> DamlResult<T> {
        T::deserialize_from(self)
    }
}

/// Marker trait for types which can be converted from a [`DamlValue`].
pub trait DamlDeserializableType: Sized {}

impl DamlDeserializableType for DamlUnit {}
impl DamlDeserializableType for DamlBool {}
impl DamlDeserializableType for DamlInt64 {}
impl DamlDeserializableType for DamlText {}
impl DamlDeserializableType for DamlParty {}
impl DamlDeserializableType for DamlContractId {}
impl DamlDeserializableType for DamlNumeric {}
impl DamlDeserializableType for DamlTimestamp {}
impl DamlDeserializableType for DamlDate {}
impl<T> DamlDeserializableType for Box<T> where T: DamlDeserializeFrom + DamlDeserializableType {}
impl<T> DamlDeserializableType for Option<T> where T: DamlDeserializeFrom + DamlDeserializableType {}
impl<T> DamlDeserializableType for Vec<T> where T: DamlDeserializeFrom + DamlDeserializableType {}
#[allow(clippy::implicit_hasher)]
impl<T> DamlDeserializableType for HashMap<String, T> where T: DamlDeserializeFrom + DamlDeserializableType {}
