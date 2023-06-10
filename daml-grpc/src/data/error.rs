use futures::io::Error;
use tonic::codegen::http;

use thiserror::Error;

/// Daml GRPC Result.
pub type DamlResult<T> = Result<T, DamlError>;

/// Daml GRPC Error.
///
/// TODO split into encode/decode errors vs request/response errors vs others
#[derive(Error, Debug)]
pub enum DamlError {
    #[error(transparent)]
    TimeoutError(Box<DamlError>),

    #[error(transparent)]
    GRPCTransportError(#[from] tonic::transport::Error),

    #[error(transparent)]
    GRPCStatusError {
        source: tonic::Status,
    },

    #[error(transparent)]
    GRPCPermissionError {
        source: tonic::Status,
    },

    #[error(transparent)]
    InvalidUriError(#[from] http::uri::InvalidUri),
    #[error(transparent)]
    StdError(#[from] Error),
    #[error("unexpected type, expected {0} but found {1}")]
    UnexpectedType(String, String),
    #[error("unknown field {0}")]
    UnknownField(String),
    #[error("list index {0} out of range")]
    ListIndexOutOfRange(usize),
    #[error("expected optional value is None")]
    MissingRequiredField,
    #[error("unexpected variant constructor, expected {0} but found {1}")]
    UnexpectedVariant(String, String),
    #[error("")]
    Other(String),
    #[error("failed conversion: {0}")]
    FailedConversion(String),
    #[error("insufficient parties")]
    InsufficientParties,
}

impl DamlError {
    pub fn new_failed_conversion(msg: impl Into<String>) -> Self {
        DamlError::FailedConversion(msg.into())
    }

    pub fn new_timeout_error(inner: DamlError) -> Self {
        DamlError::TimeoutError(Box::new(inner))
    }
}

impl From<tonic::Status> for DamlError {
    fn from(e: tonic::Status) -> Self {
        match e.code() {
            tonic::Code::Unauthenticated => DamlError::GRPCPermissionError {
                source: e,
            },
            _ => DamlError::GRPCStatusError {
                source: e,
            },
        }
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
