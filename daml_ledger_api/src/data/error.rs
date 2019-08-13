use std::error;
use std::fmt;

/// A DAML ledger error.
#[derive(Debug)]
pub enum DamlError {
    GRPC(grpcio::Error),
    ResetTimeout,
    UnexpectedType(String, String),
    UnknownField(String),
    ListIndexOutOfRange(usize),
    OptionalIsNone,
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
            DamlError::GRPC(e) => write!(fmt, "{}", (e as &dyn error::Error).description()),
            DamlError::ResetTimeout => write!(fmt, "timed out resetting ledger"),
            DamlError::UnexpectedType(expected, actual) =>
                write!(fmt, "unexpected type, expected {} but found {}", expected, actual),
            DamlError::UnknownField(name) => write!(fmt, "unknown field {}", name),
            DamlError::ListIndexOutOfRange(index) => write!(fmt, "list index {} out of range", index),
            DamlError::OptionalIsNone => write!(fmt, "optional value is None"),
            DamlError::UnexpectedVariant(expected, actual) =>
                write!(fmt, "unexpected variant constructor, expected {} but found {}", expected, actual),
            DamlError::Other(e) => write!(fmt, "{}", e),
            DamlError::FailedConversion(e) => write!(fmt, "failed conversion: {}", e),
        }
    }
}

impl error::Error for DamlError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            DamlError::GRPC(e) => Some(e),
            _ => None,
        }
    }
}

impl From<grpcio::Error> for DamlError {
    fn from(e: grpcio::Error) -> Self {
        DamlError::GRPC(e)
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
