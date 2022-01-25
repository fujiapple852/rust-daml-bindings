#![allow(clippy::used_underscore_binding)]
use thiserror::Error;

/// Represents `Daml-LF` specific errors.
#[derive(Error, Debug)]
pub enum DamlLfError {
    #[error("failed to parse Daml LF: {0}")]
    DamlLfParseError(#[from] prost::DecodeError),
    #[error("failed to parse dar file: {0}")]
    DarParseError(String),
    #[error("failed to convert Daml LF: {0}")]
    DamlLfConvertError(#[from] DamlLfConvertError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("unknown Daml LF version: {0}")]
    UnknownVersion(String),
    #[error("unsupported Daml LF version: {0}")]
    UnsupportedVersion(String),
}

impl DamlLfError {
    pub fn new_dar_parse_error(error: impl Into<String>) -> Self {
        DamlLfError::DarParseError(error.into())
    }

    pub fn new_unknown_version(version: impl Into<String>) -> Self {
        DamlLfError::UnknownVersion(version.into())
    }

    pub fn new_unsupported_version(version: impl Into<String>) -> Self {
        DamlLfError::UnsupportedVersion(version.into())
    }
}

impl From<yaml_rust::scanner::ScanError> for DamlLfError {
    fn from(e: yaml_rust::scanner::ScanError) -> Self {
        DamlLfError::DarParseError(e.to_string())
    }
}

impl From<zip::result::ZipError> for DamlLfError {
    fn from(e: zip::result::ZipError) -> Self {
        DamlLfError::DarParseError(e.to_string())
    }
}

pub type DamlLfResult<T> = Result<T, DamlLfError>;

/// Daml code generator errors.
#[derive(Debug, Error)]
pub enum DamlLfConvertError {
    /// A `DamlTypePayload::ContractId` contained more than one type argument.
    #[error("unexpected contract id type arguments")]
    UnexpectedContractIdTypeArguments,
    /// A required optional field was None.
    #[error("required field was not supplied")]
    MissingRequiredField,
    /// A Daml type not supported by the code generator was found.
    #[error("the type {0} is not currently supported")]
    UnsupportedType(String),
    /// A Daml choice was not a `DamlDataWrapper::Record`.
    #[error("choice argument was not a record")]
    UnexpectedChoiceData,
    /// A feature supported by this archive version was not used.
    #[error("Daml LF version {0} supports feature {1} but was not used (supported as of version {2})")]
    SupportedFeatureUnused(String, String, String),
    /// A feature not supported by this archive version was used.
    #[error("Daml LF version {0} does not support feature {1} (requires version {2})")]
    UnsupportedFeatureUsed(String, String, String),
    /// An unexpected `DamlDataPayload` variant was found.
    #[error("unexpected DamlDataPayload variant")]
    UnexpectedData,
    /// Expected a given `DamlTypePayload` but found a different `DamlTypePayload`
    #[error("expected type {0} but found type {1}")]
    UnexpectedType(String, String),
    /// Failed to lookup a `DamlPackagePayload` by id.
    #[error("failed to lookup a DamlPackagePayload with id {0}")]
    UnknownPackage(String),
    /// Failed to lookup a `DamlModulePayload` by id.
    #[error("failed to lookup a DamlModulePayload with id {0}")]
    UnknownModule(String),
    /// Failed to lookup a `DamlDataPayload` by id.
    #[error("failed to lookup a DamlDataPayload with id {0}")]
    UnknownData(String),
    /// Unknown PrimCon enum variant.
    #[error("unknown PrimCon enum variant {0}")]
    UnknownPrimCon(i32),
    /// Unknown BuiltinFunction enum variant.
    #[error("unknown BuiltinFunction enum variant {0}")]
    UnknownBuiltinFunction(i32),
    /// Unknown RoundingMode enum variant.
    #[error("unknown RoundingMode enum variant {0}")]
    UnknownRoundingMode(i32),
    /// Nat out of range (0..37 inclusive).
    #[error("Nat {0} out of range (0..37 inclusive)")]
    NatOutOfRange(i64),
    /// Internal error.
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Daml LF convert result.
pub type DamlLfConvertResult<T> = Result<T, DamlLfConvertError>;
