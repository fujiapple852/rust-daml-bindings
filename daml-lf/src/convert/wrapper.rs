use crate::convert::archive_payload::DamlArchivePayload;
use crate::convert::data_payload::DamlDataPayload;
#[cfg(feature = "full")]
use crate::convert::defvalue_payload::DamlDefValuePayload;
use crate::convert::module_payload::{DamlDefTypeSynPayload, DamlModulePayload};
use crate::convert::package_payload::DamlPackagePayload;

#[derive(Debug, Clone, Copy)]
pub struct PayloadElementWrapper<'a, P> {
    pub context: DamlPayloadParentContext<'a>,
    pub payload: P,
}

impl<'a, P> PayloadElementWrapper<'a, P> {
    /// Create a new `PayloadElementWrapper<Q>` for an existing `DamlPayloadDataWrapper`.
    pub const fn with_data<Q: 'a>(context: DamlPayloadParentContext<'a>, q: Q) -> PayloadElementWrapper<'a, Q> {
        PayloadElementWrapper {
            context,
            payload: q,
        }
    }

    /// Wrap a type Q in a `PayloadElementWrapper<Q>` whilst preserving context.
    pub fn wrap<Q: 'a>(self, q: &'a Q) -> PayloadElementWrapper<'a, &'a Q> {
        PayloadElementWrapper {
            context: self.context,
            payload: q,
        }
    }
}

/// A common context for wrapper types.
///
/// A common context for Daml items which exists within a given archive, package, module & data.
#[derive(Debug, Clone, Copy)]
pub struct DamlPayloadParentContext<'a> {
    pub archive: &'a DamlArchivePayload<'a>,
    pub package: &'a DamlPackagePayload<'a>,
    pub module: &'a DamlModulePayload<'a>,
    pub parent: DamlPayloadParentContextType<'a>,
}

/// DOCME
#[derive(Debug, Clone, Copy)]
pub enum DamlPayloadParentContextType<'a> {
    Data(&'a DamlDataPayload<'a>),
    DefTypeSyn(&'a DamlDefTypeSynPayload<'a>),
    #[cfg(feature = "full")]
    Value(&'a DamlDefValuePayload<'a>),
}
