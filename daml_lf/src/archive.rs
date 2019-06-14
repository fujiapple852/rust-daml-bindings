use crate::protobuf_autogen::daml_lf;
use crate::DamlLfArchivePayload;
use crate::DamlLfResult;
use bytes::IntoBuf;
use prost::Message;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// A `DAML LF` archive (aka a `dalf` file).
///
/// A `DamlLfArchive` contains a `payload` (aka "package"), a `hash` (aka "package id") of that `payload` for a
/// given `hash_function`.
#[derive(Debug)]
pub struct DamlLfArchive {
    payload: DamlLfArchivePayload,
    hash_function: DamlLfHashFunction,
    hash: String,
}

impl DamlLfArchive {
    /// Create an archive from an existing `payload`, `hash_function` and `hash`.
    ///
    /// Note that this method does not validate that the supplied `hash` is valid for the supplied `payload` and
    /// `hash_function` and thus could create an invalid archive.
    pub fn new(
        payload: impl Into<DamlLfArchivePayload>,
        hash_function: impl Into<DamlLfHashFunction>,
        hash: impl Into<String>,
    ) -> Self {
        Self {
            payload: payload.into(),
            hash_function: hash_function.into(),
            hash: hash.into(),
        }
    }

    /// Deserialize an archive from the protobuf binary representation.
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
    /// assert_eq!(&DamlLfHashFunction::SHA256, archive.hash_function());
    /// # Ok(())
    /// # }
    /// ```
    /// [`DamlLfParseError`]: crate::DamlLfError::DamlLfParseError
    /// [`UnknownVersion`]: crate::DamlLfError::UnknownVersion
    /// [`UnsupportedVersion`]: crate::DamlLfError::UnsupportedVersion
    /// [`DamlLfArchivePayload`]: DamlLfArchivePayload
    pub fn from_bytes(bytes: impl IntoBuf) -> DamlLfResult<Self> {
        let archive: daml_lf::Archive = daml_lf::Archive::decode(bytes)?;
        let payload = DamlLfArchivePayload::from_bytes(archive.payload)?;
        Ok(Self::new(payload, DamlLfHashFunction::SHA256, archive.hash))
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
    /// assert_eq!(&DamlLfHashFunction::SHA256, archive.hash_function());
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
        let mut dalf_file = File::open(dalf_path)?;
        dalf_file.read_to_end(&mut buffer)?;
        Ok(Self::from_bytes(buffer)?)
    }

    /// The payload (aka "package") contained within this archive.
    pub fn payload(&self) -> &DamlLfArchivePayload {
        &self.payload
    }

    /// The hashing function used to generate this archives `hash`.
    pub fn hash_function(&self) -> &DamlLfHashFunction {
        &self.hash_function
    }

    /// The hash of this archive (aka "package id").
    pub fn hash(&self) -> &String {
        &self.hash
    }
}

/// The hash function used to compute
#[derive(Debug, Eq, PartialEq)]
pub enum DamlLfHashFunction {
    SHA256,
}
