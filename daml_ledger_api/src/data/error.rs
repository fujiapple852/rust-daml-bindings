use futures::io::Error;
use std::error;
use std::fmt;
use tonic::codegen::http;

/// A DAML ledger error.
#[derive(Debug)]
pub enum DamlError {
    TimeoutError(Box<DamlError>),
    GRPCTransportError(tonic::transport::Error),
    GRPCStatusError(tonic::Status),
    GRPCPermissionError(tonic::Status),
    InvalidUriError(http::uri::InvalidUri),
    StdError(Error),
    UnexpectedType(String, String),
    UnknownField(String),
    ListIndexOutOfRange(usize),
    MissingRequiredField,
    UnexpectedVariant(String, String),
    Other(String),
    FailedConversion(String),
}

impl DamlError {
    pub fn new_failed_conversion(msg: impl Into<String>) -> Self {
        DamlError::FailedConversion(msg.into())
    }

    pub fn new_timeout_error(inner: DamlError) -> Self {
        DamlError::TimeoutError(Box::new(inner))
    }
}

impl fmt::Display for DamlError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DamlError::InvalidUriError(e) => write!(fmt, "{}", (e as &dyn error::Error).to_string()),
            DamlError::GRPCTransportError(e) => write!(fmt, "{}", (e as &dyn error::Error).to_string()),
            DamlError::GRPCStatusError(e) => write!(fmt, "{}", (e as &dyn error::Error).to_string()),
            DamlError::GRPCPermissionError(e) => write!(fmt, "{}", (e as &dyn error::Error).to_string()),
            DamlError::StdError(e) => write!(fmt, "{}", (e as &dyn error::Error).to_string()),
            DamlError::UnexpectedType(expected, actual) =>
                write!(fmt, "unexpected type, expected {} but found {}", expected, actual),
            DamlError::UnknownField(name) => write!(fmt, "unknown field {}", name),
            DamlError::ListIndexOutOfRange(index) => write!(fmt, "list index {} out of range", index),
            DamlError::MissingRequiredField => write!(fmt, "expected optional value is None"),
            DamlError::UnexpectedVariant(expected, actual) =>
                write!(fmt, "unexpected variant constructor, expected {} but found {}", expected, actual),
            DamlError::Other(e) => write!(fmt, "{}", e),
            DamlError::FailedConversion(e) => write!(fmt, "failed conversion: {}", e),
            DamlError::TimeoutError(e) => write!(fmt, "timeout error: {}", e),
        }
    }
}

impl error::Error for DamlError {}

impl From<tonic::Status> for DamlError {
    fn from(e: tonic::Status) -> Self {
        match e.code() {
            tonic::Code::PermissionDenied => DamlError::GRPCPermissionError(e),
            _ => DamlError::GRPCStatusError(e),
        }
    }
}

impl From<tonic::transport::Error> for DamlError {
    fn from(e: tonic::transport::Error) -> Self {
        DamlError::GRPCTransportError(e)
    }
}

impl From<http::uri::InvalidUri> for DamlError {
    fn from(e: http::uri::InvalidUri) -> Self {
        DamlError::InvalidUriError(e)
    }
}

impl From<&str> for DamlError {
    fn from(e: &str) -> Self {
        DamlError::Other(e.to_owned())
    }
}

impl From<bigdecimal::ParseBigDecimalError> for DamlError {
    fn from(e: bigdecimal::ParseBigDecimalError) -> Self {
        DamlError::FailedConversion(e.to_string())
    }
}

impl From<Error> for DamlError {
    fn from(e: Error) -> Self {
        DamlError::StdError(e)
    }
}

/// A DAML ledger result.
pub type DamlResult<T> = ::std::result::Result<T, DamlError>;
