use crate::element::DamlPackage;
use crate::lf_protobuf::com::digitalasset::daml_lf_dev::Archive;
use crate::DamlLfResult;
use crate::{convert, DamlLfArchivePayload};
use bytes::Bytes;
use prost::Message;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// The default name for an unnamed archive.
pub const DEFAULT_ARCHIVE_NAME: &str = "Unnamed";

/// A `DAML LF` archive (aka a `dalf` file).
///
/// A `DamlLfArchive` contains a `name`, a `payload` (aka "package"), a `hash` (aka "package id") of that `payload` for
/// a given `hash_function`.
#[derive(Debug, Clone)]
pub struct DamlLfArchive {
    pub name: String,
    pub payload: DamlLfArchivePayload,
    pub hash_function: DamlLfHashFunction,
    pub hash: String,
}

impl DamlLfArchive {
    /// Create an archive from an existing `payload`, `hash_function` and `hash`.
    ///
    /// Note that this method does not validate that the supplied `hash` is valid for the supplied `payload` and
    /// `hash_function` and thus could create an invalid archive.
    pub fn new(
        name: impl Into<String>,
        payload: impl Into<DamlLfArchivePayload>,
        hash_function: impl Into<DamlLfHashFunction>,
        hash: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            payload: payload.into(),
            hash_function: hash_function.into(),
            hash: hash.into(),
        }
    }

    /// Deserialize an archive from the protobuf binary representation with a default name.
    ///
    /// Deserialize the supplied protobuf `bytes` into a `DamlLfArchive`.  The embedded `payload` (bytes) will also
    /// be deserialized into a [`DamlLfArchivePayload`].
    ///
    /// # Errors
    ///
    /// If the provided bytes cannot be deserialized into an archive (or the embedded `payload` cannot be deserialized
    /// into a [`DamlLfArchivePayload`]) then [`DamlLfParseError`] will be returned.
    ///
    /// If the embedded `payload` is not of a known version then [`UnknownVersion`] will be returned.
    ///
    /// Archives of `DAML LF` `v0` are not supported and will result in a [`UnsupportedVersion`] being returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use daml_lf::{DamlLfArchive, DamlLfHashFunction};
    /// # use daml_lf::DamlLfResult;
    /// # fn main() -> DamlLfResult<()> {
    /// let buffer = Vec::<u8>::new();
    /// let archive = DamlLfArchive::from_bytes(buffer)?;
    /// assert_eq!(&DamlLfHashFunction::Sha256, archive.hash_function());
    /// # Ok(())
    /// # }
    /// ```
    /// [`DamlLfParseError`]: crate::DamlLfError::DamlLfParseError
    /// [`UnknownVersion`]: crate::DamlLfError::UnknownVersion
    /// [`UnsupportedVersion`]: crate::DamlLfError::UnsupportedVersion
    /// [`DamlLfArchivePayload`]: DamlLfArchivePayload
    pub fn from_bytes(bytes: impl Into<Bytes>) -> DamlLfResult<Self> {
        Self::from_bytes_named(DEFAULT_ARCHIVE_NAME, bytes)
    }

    /// Deserialize a named archive from the protobuf binary representation.
    ///
    /// Deserialize the supplied protobuf `bytes` into a `DamlLfArchive`.  The embedded `payload` (bytes) will also
    /// be deserialized into a [`DamlLfArchivePayload`].
    ///
    /// # Errors
    ///
    /// If the provided bytes cannot be deserialized into an archive (or the embedded `payload` cannot be deserialized
    /// into a [`DamlLfArchivePayload`]) then [`DamlLfParseError`] will be returned.
    ///
    /// If the embedded `payload` is not of a known version then [`UnknownVersion`] will be returned.
    ///
    /// Archives of `DAML LF` `v0` are not supported and will result in a [`UnsupportedVersion`] being returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use daml_lf::{DamlLfArchive, DamlLfHashFunction};
    /// # use daml_lf::DamlLfResult;
    /// # fn main() -> DamlLfResult<()> {
    /// let buffer = Vec::<u8>::new();
    /// let archive = DamlLfArchive::from_bytes_named("foo", buffer)?;
    /// assert_eq!(&DamlLfHashFunction::Sha256, archive.hash_function());
    /// assert_eq!("foo", archive.name());
    /// # Ok(())
    /// # }
    /// ```
    /// [`DamlLfParseError`]: crate::DamlLfError::DamlLfParseError
    /// [`UnknownVersion`]: crate::DamlLfError::UnknownVersion
    /// [`UnsupportedVersion`]: crate::DamlLfError::UnsupportedVersion
    /// [`DamlLfArchivePayload`]: DamlLfArchivePayload
    pub fn from_bytes_named(name: impl Into<String>, bytes: impl Into<Bytes>) -> DamlLfResult<Self> {
        let archive: Archive = Archive::decode(bytes.into())?;
        let payload = DamlLfArchivePayload::from_bytes(archive.payload)?;
        let archive_name = name.into();
        let sanitized_name = archive_name.rfind(&archive.hash).map_or(&archive_name[..], |i| &archive_name[..i - 1]);
        Ok(Self::new(sanitized_name, payload, DamlLfHashFunction::Sha256, archive.hash))
    }

    /// Read and parse an archive from a `dalf` file.
    ///
    /// # Errors
    ///
    /// If the provided file cannot be read an [`IOError`] will be returned which contains the underlying IO error.
    ///
    /// If the contents of the file cannot be deserialized into an archive (or the embedded `payload` cannot be
    /// deserialized into a [`DamlLfArchivePayload`]) then [`DamlLfParseError`] will be returned.
    ///
    /// If the embedded `payload` is not of a known version then [`UnknownVersion`] will be returned.
    ///
    /// Archives of `DAML LF` `v0` are not supported and will result in a [`UnsupportedVersion`] being returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use daml_lf::{DamlLfArchive, DamlLfHashFunction};
    /// # use daml_lf::DamlLfResult;
    /// # fn main() -> DamlLfResult<()> {
    /// let archive = DamlLfArchive::from_file("Example.dalf")?;
    /// assert_eq!(&DamlLfHashFunction::Sha256, archive.hash_function());
    /// assert_eq!("Example", archive.name());
    /// # Ok(())
    /// # }
    /// ```
    /// [`IOError`]: crate::DamlLfError::IOError
    /// [`DamlLfParseError`]: crate::DamlLfError::DamlLfParseError
    /// [`UnknownVersion`]: crate::DamlLfError::UnknownVersion
    /// [`UnsupportedVersion`]: crate::DamlLfError::UnsupportedVersion
    /// [`DamlLfArchivePayload`]: DamlLfArchivePayload
    pub fn from_file(dalf_path: impl AsRef<Path>) -> DamlLfResult<Self> {
        let mut buffer = Vec::new();
        let mut dalf_file = File::open(dalf_path.as_ref())?;
        dalf_file.read_to_end(&mut buffer)?;
        let archive_name_stem = dalf_path.as_ref().file_stem().and_then(OsStr::to_str).unwrap_or(DEFAULT_ARCHIVE_NAME);
        Self::from_bytes_named(archive_name_stem, buffer)
    }

    /// Create a [`DamlArchive`] from a [`DamlLfArchive`] and apply it to `f`.
    ///
    /// See [`DarFile::apply`] for details.
    ///
    /// [`DamlArchive`]: crate::element::DamlArchive
    /// [`DarFile::apply`]: crate::dar::DarFile::apply
    pub fn apply<R, F>(&self, f: F) -> DamlLfResult<R>
    where
        F: FnOnce(&DamlPackage<'_>) -> R,
    {
        convert::apply_dalf(self, f)
    }

    /// The name of this archive.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The payload (aka "package") contained within this archive.
    pub const fn payload(&self) -> &DamlLfArchivePayload {
        &self.payload
    }

    /// The hashing function used to generate this archives `hash`.
    pub const fn hash_function(&self) -> &DamlLfHashFunction {
        &self.hash_function
    }

    /// The hash of this archive (aka "package id").
    pub fn hash(&self) -> &str {
        &self.hash
    }
}

/// The hash function used to compute
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DamlLfHashFunction {
    Sha256,
}
