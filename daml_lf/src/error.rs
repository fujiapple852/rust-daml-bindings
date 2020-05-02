use thiserror::Error;

/// Represents `DAML-LF` specific errors.
#[derive(Error, Debug)]
pub enum DamlLfError {
    #[error("failed to parse DAML LF: {0}")]
    DamlLfParseError(#[from] prost::DecodeError),

    #[error("failed to encode DAML LF: {0}")]
    DamlLfEncodingError(#[from] prost::EncodeError),

    #[error("failed to parse dar file: {0}")]
    DarParseError(String),
    #[error("failed to convert DAML LF: {0}")]
    DamlLfConvertError(#[from] DamlLfConvertError),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("unknown DAML LF version: {0}")]
    UnknownVersion(String),
    #[error("unsupported DAML LF version: {0}")]
    UnsupportedVersion(String),
}

// TODO have manifest error? lots of manifest errors show as DarParseError
// TODO have zip error?  need to suppot read and write so parse error not right

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

/// DAML code generator errors.
#[derive(Debug, Error)]
pub enum DamlLfConvertError {
    /// A `DamlTypePayload::ContractId` contained more than one type argument.
    #[error("unexpected contract id type arguments")]
    UnexpectedContractIdTypeArguments,
    /// A required optional field was None.
    #[error("required field was not supplied")]
    MissingRequiredField,
    /// A DAML type not supported by the code generator was found.
    #[error("the type {0} is not currently supported")]
    UnsupportedType(String),
    /// A DAML choice was not a `DamlDataWrapper::Record`.
    #[error("choice argument was not a record")]
    UnexpectedChoiceData,
    /// A feature supported by this archive version was not used.
    #[error("DAML LF version {0} supports feature {1} but was not used (supported as of version {2})")]
    SupportedFeatureUnused(String, String, String),
    /// A feature not supported by this archive version was used.
    #[error("DAML LF version {0} does not support feature {1} (requires version {2})")]
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
    /// Nat out of range (0..37 inclusive).
    #[error("Nat {0} out of range (0..37 inclusive)")]
    NatOutOfRange(i64),
}

/// DAML LF convert result.
pub type DamlLfConvertResult<T> = Result<T, DamlLfConvertError>;
