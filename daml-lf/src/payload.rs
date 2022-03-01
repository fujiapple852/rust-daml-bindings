use crate::element::DamlPackage;
use crate::error::{DamlLfError, DamlLfResult};
use crate::lf_protobuf::com::daml::daml_lf::archive_payload::Sum;
use crate::lf_protobuf::com::daml::daml_lf::ArchivePayload;
use crate::lf_protobuf::com::daml::daml_lf_1;
use crate::lf_protobuf::com::daml::daml_lf_1::module::Name;
use crate::{convert, LanguageV1MinorVersion, LanguageVersion};
use bytes::Bytes;
use itertools::Itertools;
use prost::Message;
use std::convert::TryFrom;

/// A `Daml LF` archive payload (aka "package").
///
/// A `Daml LF` archive payload contains a `package` and a `language_version`.
#[derive(Debug, Clone)]
pub struct DamlLfArchivePayload {
    pub language_version: LanguageVersion,
    pub package: DamlLfPackage,
}

impl DamlLfArchivePayload {
    /// Create a `DamlLfArchivePayload` from an existing [`DamlLfPackage`] and `language_version`.
    ///
    /// [`DamlLfPackage`]: enum.DamlLfPackage.html
    pub const fn new(language_version: LanguageVersion, package: DamlLfPackage) -> Self {
        Self {
            language_version,
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
    /// Archives of `Daml LF` `v0` are not supported and will result in a [`UnsupportedVersion`]
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
    /// ../daml-grpc/service/daml_package_service/struct.DamlPackageService.html#method.get_package
    /// [`get_package_sync`]:
    /// ../daml-grpc/service/daml_package_service/struct.DamlPackageService.html#method.get_package_sync
    /// [`payload()`]: ../../doc/daml-grpc/data/package/struct.DamlPackage.html#method.payload
    /// [`DamlPackage`]: ../../doc/daml-grpc/data/package/struct.DamlPackage.html
    /// [`UnsupportedVersion`]: DamlLfError::UnsupportedVersion
    /// [`DamlLfParseError`]: DamlLfError::DamlLfParseError
    /// [`UnknownVersion`]: DamlLfError::UnknownVersion
    pub fn from_bytes(payload_buffer: impl Into<Bytes>) -> DamlLfResult<Self> {
        let payload: ArchivePayload = ArchivePayload::decode(payload_buffer.into())?;
        match payload.sum {
            Some(Sum::DamlLf1(p)) => Ok(Self::new(
                LanguageVersion::new_v1(LanguageV1MinorVersion::try_from(payload.minor.as_str())?),
                DamlLfPackage::V1(p),
            )),
            _ => Err(DamlLfError::new_unknown_version("none")),
        }
    }

    /// Create a [`DamlArchive`] from this [`DamlLfArchivePayload`] and apply it to `f`.
    ///
    /// See [`DarFile::apply`] for details.
    ///
    /// [`DamlArchive`]: crate::element::DamlArchive
    /// [`DarFile::apply`]: crate::dar::DarFile::apply
    pub fn apply<R, F>(self, f: F) -> DamlLfResult<R>
    where
        F: FnOnce(&DamlPackage<'_>) -> R,
    {
        convert::apply_payload(self, f)
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
                Some(name) => self.decode_dotted_name(name) == module,
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
            DamlLfPackage::V1(package) =>
                package.modules.iter().filter_map(|m| m.name.as_ref().map(|dn| self.decode_dotted_name(dn))).collect(),
        }
    }

    /// The `language_version` version of this `payload`.
    pub const fn language_version(&self) -> &LanguageVersion {
        &self.language_version
    }

    /// The package embedded in this `payload`.
    pub const fn package(&self) -> &DamlLfPackage {
        &self.package
    }

    fn decode_dotted_name(&self, name: &Name) -> String {
        match &self.package {
            DamlLfPackage::V1(package) => match name {
                Name::NameInternedDname(i) => package
                    .interned_dotted_names
                    .get(*i as usize)
                    .map(|dn| {
                        dn.segments_interned_str
                            .iter()
                            .map(|&i| package.interned_strings.get(i as usize).expect("Package.interned_strings"))
                            .join(".")
                    })
                    .expect("Package.interned_dotted_names"),
                Name::NameDname(dn) => dn.segments.join("."),
            },
        }
    }
}

/// The supported `Daml LF` package formats.
#[derive(Debug, Clone)]
pub enum DamlLfPackage {
    V1(daml_lf_1::Package),
}
