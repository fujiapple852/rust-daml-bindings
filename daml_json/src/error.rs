use crate::util::{NotSingleError, RequiredError};
use thiserror::Error;

/// Daml JSON Result.
pub type DamlJsonResult<T> = Result<T, DamlJsonError>;

/// Daml JSON Error.
#[derive(Error, Debug)]
pub enum DamlJsonError {
    #[error("DamlJsonError: codec error: {0}")]
    CodecError(#[from] DamlJsonCodecError),
    #[error("DamlJsonError: GRPC error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    MissingRequiredField(#[from] RequiredError),
    #[error("DamlJsonError: error response: {0}, {1}")]
    ErrorResponse(u16, String),
    #[error("DamlJsonError: url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("DamlJsonError: unhandled http response code: {0}")]
    UnhandledHttpResponse(String),
}

/// Daml JSON Request/Response Converter Result.
pub type DamlJsonReqConResult<T> = Result<T, DamlJsonReqConError>;

/// Daml JSON Request/Response Converter Error.
#[derive(Error, Debug)]
pub enum DamlJsonReqConError {
    #[error("DamlJsonError: codec error: {0}")]
    CodecError(#[from] DamlJsonCodecError),
    #[error("DamlJsonError: GRPC error: {0}")]
    DamlGrpcError(#[from] daml_grpc::data::DamlError),
    #[error("DamlJsonError: invalid template id format: {0}")]
    TemplateIdFormatError(String),
    #[error("DamlJsonError: unknown template id: {0}")]
    UnknownTemplateId(String),
    #[error("DamlJsonError: template {0} exists in multiple packages: {1:#?}")]
    MultipleMatchingTemplates(String, Vec<String>),
    #[error("DamlJsonError: Expected a template for: {0}")]
    ExpectedTemplateError(String),
    #[error("DamlJsonError: template does not have a contract key: {0}")]
    TemplateNoKeyError(String),
    #[error("DamlJsonError: expected exactly 1 GRPC event")]
    UnexpectedGrpcEvent,
    #[error("DamlJsonError: Transaction tree did not contain an exercised event")]
    MissingExercisedEvent,
}

/// Daml JSON Codec Result.
pub type DamlJsonCodecResult<T> = Result<T, DamlJsonCodecError>;

/// Daml JSON Codec Error.
#[derive(Error, Debug)]
pub enum DamlJsonCodecError {
    #[error("failed to process Daml LF: {0}")]
    DamlLfError(#[from] daml_lf::DamlLfError),
    #[error("failed to parse numeric from string: {0}")]
    NumericParseError(#[from] bigdecimal::ParseBigDecimalError),
    #[error("failed to parse int64 from string: {0}")]
    Int64ParseError(#[from] std::num::ParseIntError),
    #[error("failed to parse date or datetime from string: {0}")]
    DateParseError(#[from] chrono::format::ParseError),
    #[error(transparent)]
    MissingRequiredField(#[from] RequiredError),
    #[error(transparent)]
    UnexpectedListEntries(#[from] NotSingleError),
    #[error("expected JSON type {0} but found type {1}")]
    UnexpectedJsonType(String, String),
    #[error("record object did not contain expected field {0}")]
    MissingJsonRecordObjectField(String),
    #[error("record array did not contain expected field {1} at index {0}")]
    MissingJsonRecordArrayField(usize, String),
    #[error("unknown variant constructor {0}")]
    UnknownVariantConstructor(String),
    #[error("unknown enum constructor {0}")]
    UnknownEnumConstructor(String),
    #[error("expected empty record for Unit type")]
    UnexpectedUnitData,
    #[error("expected an array with either zero or one entry")]
    UnexpectedOptionalArrayLength,
    #[error("duplicate genmap keys")]
    DuplicateGenMapKeys,
    #[error("expected exactly two types for genmap")]
    UnexpectedGenMapTypes,
    #[error("unsupported Daml type {0}")]
    UnsupportedDamlType(String),
    #[error("Data item {0} not found in archive")]
    DataNotFound(String),
}

/// Daml JSON Schema Codec Result.
pub type DamlJsonSchemaCodecResult<T> = Result<T, DamlJsonSchemaCodecError>;

/// Daml JSON Schema Codec Error.
#[derive(Error, Debug)]
pub enum DamlJsonSchemaCodecError {
    #[error("failed to process Daml LF: {0}")]
    DamlLfError(#[from] daml_lf::DamlLfError),
    #[error(transparent)]
    MissingRequiredField(#[from] RequiredError),
    #[error(transparent)]
    UnexpectedListEntries(#[from] NotSingleError),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error("Data item {0} not found in archive")]
    DataNotFound(String),
    #[error("unsupported Daml type {0} in JSON Schema")]
    UnsupportedDamlType(String),
    #[error("Daml type {0} in is not serializable")]
    NotSerializableDamlType(String),
    #[error("Daml type variable '{0}' not found in type arguments")]
    TypeVarNotFoundInArgs(String),
    #[error("Daml type variable '{0}' not found in type parameters")]
    TypeVarNotFoundInParams(String),
}
