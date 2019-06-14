use crate::data::DamlError;
use crate::grpc_protobuf_autogen::ledger_offset::LedgerOffset;
use crate::grpc_protobuf_autogen::ledger_offset::LedgerOffset_LedgerBoundary;
use crate::grpc_protobuf_autogen::ledger_offset::LedgerOffset_oneof_value;
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

#[derive(Debug)]
pub enum DamlLedgerOffsetType {
    Unbounded,
    Bounded(DamlLedgerOffset),
}

#[derive(PartialEq, Eq, Debug)]
pub enum DamlLedgerOffset {
    Absolute(u64),
    Boundary(DamlLedgerOffsetBoundary),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum DamlLedgerOffsetBoundary {
    Begin,
    End,
}

impl PartialOrd for DamlLedgerOffset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (DamlLedgerOffset::Absolute(a1), DamlLedgerOffset::Absolute(a2)) => Some(a1.cmp(a2)),
            (DamlLedgerOffset::Boundary(b1), DamlLedgerOffset::Boundary(b2)) => Some(b1.cmp(&b2)),
            _ => None,
        }
    }
}

impl TryFrom<LedgerOffset> for DamlLedgerOffset {
    type Error = DamlError;

    fn try_from(offset: LedgerOffset) -> Result<Self, Self::Error> {
        match offset.value {
            Some(sum) => {
                let convert = |sum| {
                    Ok(match sum {
                        LedgerOffset_oneof_value::absolute(abs) =>
                            DamlLedgerOffset::Absolute(u64::from_str(&abs).map_err(|e| {
                                DamlError::new_failed_conversion(format!("invalid ledger offset: {}", e.to_string()))
                            })?),
                        LedgerOffset_oneof_value::boundary(LedgerOffset_LedgerBoundary::LEDGER_BEGIN) =>
                            DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::Begin),
                        LedgerOffset_oneof_value::boundary(LedgerOffset_LedgerBoundary::LEDGER_END) =>
                            DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::End),
                    })
                };
                convert(sum)
            },
            None => Err(DamlError::OptionalIsNone),
        }
    }
}

impl From<DamlLedgerOffset> for LedgerOffset {
    fn from(daml_ledger_offset: DamlLedgerOffset) -> Self {
        let mut offset = Self::new();
        match daml_ledger_offset {
            DamlLedgerOffset::Absolute(s) => offset.set_absolute(s.to_string()),
            DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::Begin) =>
                offset.set_boundary(LedgerOffset_LedgerBoundary::LEDGER_BEGIN),
            DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::End) =>
                offset.set_boundary(LedgerOffset_LedgerBoundary::LEDGER_END),
        }
        offset
    }
}
