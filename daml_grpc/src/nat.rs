use crate::data::value::DamlValue;
use crate::data::DamlResult;
use crate::serialize::{DamlDeserializableType, DamlDeserializeFrom, DamlSerializableType, DamlSerializeFrom};

///
pub trait Nat {
    fn nat() -> u8;
}

///
macro_rules! make_nat {
    ($name:ident, $n:literal) => {
        make_nat_struct! {$name, $n}
        make_nat_serializable_type! {$name}
        make_nat_deserializable_type! {$name}
        make_nat_serialize_from! {$name}
        make_nat_deserialize_from! {$name}
    };
}

macro_rules! make_nat_struct {
    ($name:ident, $n:literal) => {
        #[derive(Debug, Eq, PartialEq, Clone)]
        pub struct $name {}
        impl Nat for $name {
            fn nat() -> u8 {
                $n
            }
        }
    };
}

macro_rules! make_nat_serializable_type {
    ($name:ident) => {
        impl DamlSerializableType for $name {}
    };
}

macro_rules! make_nat_deserializable_type {
    ($name:ident) => {
        impl DamlDeserializableType for $name {}
    };
}

macro_rules! make_nat_serialize_from {
    ($name:ident) => {
        impl DamlSerializeFrom<$name> for DamlValue {
            fn serialize_from(_: $name) -> DamlValue {
                Self::new_unit()
            }
        }
    };
}

macro_rules! make_nat_deserialize_from {
    ($name:ident) => {
        impl DamlDeserializeFrom for $name {
            fn deserialize_from(_: DamlValue) -> DamlResult<Self> {
                Ok($name {})
            }
        }
    };
}

make_nat! {Nat0, 0}
make_nat! {Nat1, 1}
make_nat! {Nat2, 2}
make_nat! {Nat3, 3}
make_nat! {Nat4, 4}
make_nat! {Nat5, 5}
make_nat! {Nat6, 6}
make_nat! {Nat7, 7}
make_nat! {Nat8, 8}
make_nat! {Nat9, 9}
make_nat! {Nat10, 10}
make_nat! {Nat11, 11}
make_nat! {Nat12, 12}
make_nat! {Nat13, 13}
make_nat! {Nat14, 14}
make_nat! {Nat15, 15}
make_nat! {Nat16, 16}
make_nat! {Nat17, 17}
make_nat! {Nat18, 18}
make_nat! {Nat19, 19}
make_nat! {Nat20, 20}
make_nat! {Nat21, 21}
make_nat! {Nat22, 22}
make_nat! {Nat23, 23}
make_nat! {Nat24, 24}
make_nat! {Nat25, 25}
make_nat! {Nat26, 26}
make_nat! {Nat27, 27}
make_nat! {Nat28, 28}
make_nat! {Nat29, 29}
make_nat! {Nat30, 30}
make_nat! {Nat31, 31}
make_nat! {Nat32, 32}
make_nat! {Nat33, 33}
make_nat! {Nat34, 34}
make_nat! {Nat35, 35}
make_nat! {Nat36, 36}
make_nat! {Nat37, 37}
