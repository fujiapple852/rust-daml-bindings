use crate::convert::archive::wrapper::payload::*;
use crate::convert::archive::wrapper::*;

#[derive(Debug, Clone, Copy)]
pub struct DamlArchiveWrapper<'a> {
    pub payload: &'a DamlArchivePayload<'a>,
}

impl<'a> DamlArchiveWrapper<'a> {
    pub fn wrap(payload: &'a DamlArchivePayload) -> Self {
        Self {
            payload,
        }
    }

    pub fn packages(self) -> impl Iterator<Item = DamlPackageWrapper<'a>> {
        self.payload.packages.values().map(move |package| DamlPackageWrapper::wrap(self.payload, package))
    }
}
