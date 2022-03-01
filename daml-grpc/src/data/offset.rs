use crate::data::DamlError;
use crate::grpc_protobuf::com::daml::ledger::api::v1::ledger_offset::{LedgerBoundary, Value};
use crate::grpc_protobuf::com::daml::ledger::api::v1::LedgerOffset;
use crate::util::Required;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::str::FromStr;

// TODO support alternative ledger offset string formats.
//
// From proto comments:
//
// Absolute values are acquired by reading the transactions in the stream.
// The offsets can be compared. The format may vary between implementations.
// It is either:
// * a string representing an ever-increasing integer
// * a composite string containing <block-hash>-<block-height>-<event-id>; ordering
// requires comparing numerical values of the second, then the third element.

#[derive(Debug, Clone)]
pub enum DamlLedgerOffsetType {
    Unbounded,
    Bounded(DamlLedgerOffset),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum DamlLedgerOffset {
    Absolute(u64),
    Boundary(DamlLedgerOffsetBoundary),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum DamlLedgerOffsetBoundary {
    Begin,
    End,
}

impl PartialOrd for DamlLedgerOffset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (DamlLedgerOffset::Absolute(a1), DamlLedgerOffset::Absolute(a2)) => Some(a1.cmp(a2)),
            (DamlLedgerOffset::Boundary(b1), DamlLedgerOffset::Boundary(b2)) => Some(b1.cmp(b2)),
            _ => None,
        }
    }
}

impl TryFrom<LedgerOffset> for DamlLedgerOffset {
    type Error = DamlError;

    fn try_from(offset: LedgerOffset) -> Result<Self, Self::Error> {
        match offset.value.req()? {
            Value::Absolute(abs) => match u64::from_str(&abs) {
                Ok(v) => Ok(DamlLedgerOffset::Absolute(v)),
                Err(e) => Err(DamlError::new_failed_conversion(format!("invalid ledger offset: {}", e))),
            },
            Value::Boundary(i) => match LedgerBoundary::from_i32(i) {
                Some(LedgerBoundary::LedgerBegin) => Ok(DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::Begin)),
                Some(LedgerBoundary::LedgerEnd) => Ok(DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::End)),
                None => Err(DamlError::new_failed_conversion(format!("unknown ledger boundary offset: {}", i))),
            },
        }
    }
}

impl From<DamlLedgerOffset> for LedgerOffset {
    fn from(daml_ledger_offset: DamlLedgerOffset) -> Self {
        LedgerOffset {
            value: match daml_ledger_offset {
                DamlLedgerOffset::Absolute(s) => Some(Value::Absolute(s.to_string())),
                DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::Begin) =>
                    Some(Value::Boundary(LedgerBoundary::LedgerBegin as i32)),
                DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::End) =>
                    Some(Value::Boundary(LedgerBoundary::LedgerEnd as i32)),
            },
        }
    }
}
