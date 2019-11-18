use crate::error::{DamlLfError, DamlLfResult};
use crate::protobuf_autogen::{daml_lf, daml_lf_1};
use bytes::IntoBuf;
use prost::Message;

/// A `DAML LF` archive payload (aka "package").
///
/// A `DAML LF` archive payload contains a `package` and a `minor` version.
#[derive(Debug, Clone)]
pub struct DamlLfArchivePayload {
    pub minor: String,
    pub package: DamlLfPackage,
}

impl DamlLfArchivePayload {
    /// Create a `DamlLfArchivePayload` from an existing [`DamlLfPackage`] and `minor` version.
    ///
    /// [`DamlLfPackage`]: enum.DamlLfPackage.html
    pub fn new(minor: String, package: DamlLfPackage) -> Self {
        Self {
            minor,
            package,
        }
    }

    /// Create a `DamlLfArchivePayload` from a serialized protobuf byte buffer.
    ///
    /// This method is suitable for use with the bytes returned by the [`payload()`] method of [`DamlPackage`] which is
    /// returned by the [`get_package`] and [`get_package_sync`] methods.
    ///
    /// # Errors
    ///
    /// If the `payload_buffer` cannot be deserialized into a `DamlLfArchivePayload` then
    /// [`DamlLfParseError`] will be returned.
    ///
    /// If the deserialized `DamlLfArchivePayload` is not of a known version then [`UnknownVersion`]
    /// will be returned.
    ///
    /// Archives of `DAML LF` `v0` are not supported and will result in a [`UnsupportedVersion`]
    /// being returned.
    ///
    /// ```no_run
    /// # use daml_lf::DamlLfResult;
    /// # use daml_lf::DamlLfArchivePayload;
    /// # fn main() -> DamlLfResult<()> {
    /// let buffer = Vec::<u8>::new();
    /// let payload = DamlLfArchivePayload::from_bytes(buffer)?;
    /// assert_eq!(true, payload.contains_module("PingPong"));
    /// # Ok(())
    /// # }
    /// ```
    /// [`get_package`]:
    /// ../daml_ledger_api/service/daml_package_service/struct.DamlPackageService.html#method.get_package
    /// [`get_package_sync`]:
    /// ../daml_ledger_api/service/daml_package_service/struct.DamlPackageService.html#method.get_package_sync
    /// [`payload()`]: ../../doc/daml_ledger_api/data/package/struct.DamlPackage.html#method.payload
    /// [`DamlPackage`]: ../../doc/daml_ledger_api/data/package/struct.DamlPackage.html
    /// [`UnsupportedVersion`]: DamlLfError::UnsupportedVersion
    /// [`DamlLfParseError`]: DamlLfError::DamlLfParseError
    /// [`UnknownVersion`]: DamlLfError::UnknownVersion
    pub fn from_bytes(payload_buffer: impl IntoBuf) -> DamlLfResult<Self> {
        let payload: daml_lf::ArchivePayload = daml_lf::ArchivePayload::decode(payload_buffer)?;
        let package = match payload.sum {
            Some(daml_lf::archive_payload::Sum::DamlLf0(_)) => Err(DamlLfError::UnsupportedVersion),
            Some(daml_lf::archive_payload::Sum::DamlLf1(p)) => Ok(DamlLfPackage::V1(p)),
            _ => Err(DamlLfError::UnknownVersion),
        }?;
        Ok(Self::new(payload.minor, package))
    }

    /// Returns true if the `package` within this `DamlLfArchivePayload` contains `module`, flase otherwise.
    ///
    /// The supplied `module` name is assumed to be in `DottedName` format, i.e. `TopModule.SubModule.Module`.
    ///
    /// ```no_run
    /// # use daml_lf::DamlLfResult;
    /// # use daml_lf::DamlLfArchivePayload;
    /// # fn main() -> DamlLfResult<()> {
    /// let buffer = Vec::<u8>::new();
    /// let payload = DamlLfArchivePayload::from_bytes(buffer)?;
    /// assert_eq!(true, payload.contains_module("PingPong"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn contains_module(&self, module: &str) -> bool {
        match &self.package {
            DamlLfPackage::V1(package) => package.modules.iter().any(|m| match &m.name {
                Some(dn) => dn.segments.join(".") == module,
                _ => false,
            }),
        }
    }

    /// Returns a list of all module names with the `package` contained within this `DamlLfArchivePayload`.
    ///
    /// The returned module names are strings in `DottedName` format, i.e. `TopModule.SubModule.Module`.
    ///
    /// ```no_run
    /// # use daml_lf::DamlLfResult;
    /// # use daml_lf::DamlLfArchivePayload;
    /// # fn main() -> DamlLfResult<()> {
    /// let buffer = Vec::<u8>::new();
    /// let payload = DamlLfArchivePayload::from_bytes(buffer)?;
    /// assert_eq!(vec!["PingPong", "Module1.Module2"], payload.list_modules());
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_modules(&self) -> Vec<String> {
        match &self.package {
            DamlLfPackage::V1(package) => package
                .modules
                .iter()
                .filter_map(|m| match &m.name {
                    Some(dn) => Some(dn.segments.join(".")),
                    _ => None,
                })
                .collect(),
        }
    }

    /// The minor version of this `payload`.
    pub fn minor(&self) -> &str {
        &self.minor
    }

    /// The package embedded in this `payload`.
    pub fn package(&self) -> &DamlLfPackage {
        &self.package
    }
}

/// The supported `DAML LF` package formats.
#[derive(Debug, Clone)]
pub enum DamlLfPackage {
    V1(daml_lf_1::Package),
}
