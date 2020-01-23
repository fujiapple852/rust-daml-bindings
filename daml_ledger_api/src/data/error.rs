use std::error;
use std::fmt;
use tonic::codegen::http;

/// A DAML ledger error.
#[derive(Debug)]
pub enum DamlError {
    GRPCTransportError(tonic::transport::Error),
    GRPCStatusError(tonic::Status),
    InvalidUriError(http::uri::InvalidUri),
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
}

impl fmt::Display for DamlError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DamlError::InvalidUriError(e) => write!(fmt, "{}", (e as &dyn error::Error).description()),
            DamlError::GRPCTransportError(e) => write!(fmt, "{}", (e as &dyn error::Error).description()),
            DamlError::GRPCStatusError(e) => write!(fmt, "{}", (e as &dyn error::Error).description()),
            DamlError::UnexpectedType(expected, actual) =>
                write!(fmt, "unexpected type, expected {} but found {}", expected, actual),
            DamlError::UnknownField(name) => write!(fmt, "unknown field {}", name),
            DamlError::ListIndexOutOfRange(index) => write!(fmt, "list index {} out of range", index),
            DamlError::MissingRequiredField => write!(fmt, "expected optional value is None"),
            DamlError::UnexpectedVariant(expected, actual) =>
                write!(fmt, "unexpected variant constructor, expected {} but found {}", expected, actual),
            DamlError::Other(e) => write!(fmt, "{}", e),
            DamlError::FailedConversion(e) => write!(fmt, "failed conversion: {}", e),
        }
    }
}

impl error::Error for DamlError {}

impl From<tonic::Status> for DamlError {
    fn from(e: tonic::Status) -> Self {
        DamlError::GRPCStatusError(e)
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

/// A DAML ledger result.
pub type DamlResult<T> = ::std::result::Result<T, DamlError>;
