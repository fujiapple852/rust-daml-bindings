use crate::convert::archive_payload::DamlArchivePayload;
use crate::convert::data_payload::DamlDataPayload;
use crate::convert::module_payload::DamlModulePayload;
use crate::convert::package_payload::DamlPackagePayload;

#[derive(Debug, Clone, Copy)]
pub struct PayloadElementWrapper<'a, P> {
    pub context: DamlPayloadDataWrapper<'a>,
    pub payload: P,
}

impl<'a, P> PayloadElementWrapper<'a, P> {
    /// Create a new `PayloadElementWrapper<Q>` for an existing `DamlPayloadDataWrapper`.
    pub fn with_data<Q: 'a>(context: DamlPayloadDataWrapper<'a>, q: Q) -> PayloadElementWrapper<'a, Q> {
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
/// A common context for DAML items which exists within a given archive, package, module & data.
#[derive(Debug, Clone, Copy)]
pub struct DamlPayloadDataWrapper<'a> {
    pub archive: &'a DamlArchivePayload<'a>,
    pub package: &'a DamlPackagePayload<'a>,
    pub module: &'a DamlModulePayload<'a>,
    pub data: &'a DamlDataPayload<'a>,
}
