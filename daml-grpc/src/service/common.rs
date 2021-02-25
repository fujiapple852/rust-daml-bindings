use std::fmt::Debug;
use tonic::metadata::MetadataValue;

use tonic::{Code, Request};
use tracing::trace;

use crate::data::{DamlError, DamlResult};

pub fn make_request<T>(payload: T, auth_token: Option<&str>) -> DamlResult<Request<T>> {
    let mut request = Request::new(payload);
    if let Some(token) = auth_token {
        let token =
            MetadataValue::from_str(&format!("Bearer {}", token)).map_err(|e| DamlError::Other(e.to_string()))?;
        request.metadata_mut().insert("authorization", token);
    }
    Ok(request)
}

// TODO
pub fn make_request2<T, E: std::error::Error>(
    payload: T,
    auth_token: Option<&str>,
) -> Result<Request<T>, GrpcApiError<E>> {
    let mut request = Request::new(payload);
    if let Some(token) = auth_token {
        let token = format!("Bearer {}", token);
        let token_meta = MetadataValue::from_str(&token)
            .map_err(|e| GrpcApiError::RequestError(format!("{0}: {1}", e.to_string(), token)))?;
        request.metadata_mut().insert("authorization", token_meta);
    }
    Ok(request)
}

pub fn trace_item<T: Debug>(t: &T) {
    trace!(?t)
}

/// A Generic GRPC API Error.
#[derive(thiserror::Error, Debug)]
pub enum GrpcApiError<E: std::error::Error + Debug + 'static> {
    #[error("the API request could not be created: {0}")]
    RequestError(String),
    #[error(transparent)]
    EncodeGrpcError(DamlError),
    #[error(transparent)]
    DecodeGrpcError(DamlError),
    #[error("ApiError: {error}, message={message}")]
    ApiError {
        error: E,
        message: String,
    },
}

impl<E: std::error::Error + Debug + 'static> GrpcApiError<E> {
    pub fn new(error: E, message: impl Into<String>) -> Self {
        GrpcApiError::ApiError {
            error,
            message: message.into(),
        }
    }
}

/// TODO Assume a DamlError is just a conversion error (response only?) for now, will refactor into separate error
/// types?
impl<E: std::error::Error + Debug + 'static> From<DamlError> for GrpcApiError<E> {
    fn from(e: DamlError) -> Self {
        match e {
            // _ => GrpcApiError::DecodeResponseError(e),
            // DamlError::TimeoutError(_) => {}
            // DamlError::GRPCTransportError(_) => {}
            // DamlError::GRPCStatusError { .. } => {}
            // DamlError::GRPCPermissionError { .. } => {}
            // DamlError::InvalidUriError(_) => {}
            // DamlError::StdError(_) => {}
            // DamlError::UnexpectedType(_, _) => {}
            // DamlError::UnknownField(_) => {}
            // DamlError::ListIndexOutOfRange(_) => {}
            // DamlError::MissingRequiredField => {}
            // DamlError::UnexpectedVariant(_, _) => {}
            // DamlError::Other(_) => {}
            DamlError::FailedConversion(_) => GrpcApiError::DecodeGrpcError(e),
            // DamlError::InsufficientParties => {}
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub enum GrpcCode {
    /// The operation completed successfully.
    Ok = 0,
    /// The operation was cancelled.
    Cancelled = 1,
    /// Unknown error.
    Unknown = 2,
    /// Client specified an invalid argument.
    InvalidArgument = 3,
    /// Deadline expired before operation could complete.
    DeadlineExceeded = 4,
    /// Some requested entity was not found.
    NotFound = 5,
    /// Some entity that we attempted to create already exists.
    AlreadyExists = 6,
    /// The caller does not have permission to execute the specified operation.
    PermissionDenied = 7,
    /// Some resource has been exhausted.
    ResourceExhausted = 8,
    /// The system is not in a state required for the operation's execution.
    FailedPrecondition = 9,
    /// The operation was aborted.
    Aborted = 10,
    /// Operation was attempted past the valid range.
    OutOfRange = 11,
    /// Operation is not implemented or not supported.
    Unimplemented = 12,
    /// Internal error.
    Internal = 13,
    /// The service is currently unavailable.
    Unavailable = 14,
    /// Unrecoverable data loss or corruption.
    DataLoss = 15,
    /// The request does not have valid authentication credentials
    Unauthenticated = 16,
}

impl From<Code> for GrpcCode {
    fn from(_: Code) -> Self {
        unimplemented!()
    }
}

// create a GRPC error types
#[macro_export]
macro_rules! make_grpc_error_type {
    ($code_id:ident, $error_id:ident, $result_id:ident, $($entry_id:ident => $msg_txt:literal)|*) => {
        #[derive(thiserror::Error, Debug)]
        pub enum $code_id {
            $(
                #[error($msg_txt)]
                $entry_id,
            )*
            #[error("the request does not include a valid access token")]
            Unauthenticated,
            #[error("the claims in the token are insufficient to perform a given operation")]
            PermissionDenied,
            #[error("an unexpected error occurred: {0:#?}")]
            Other(GrpcCode),
        }
        pub type $error_id = GrpcApiError<$code_id>;
        impl From<tonic::Status> for $error_id {
            fn from(status: tonic::Status) -> Self {
                match status.code() {
                    $(
                        tonic::Code::$entry_id => $error_id::new($code_id::$entry_id, status.message()),
                    )*
                    tonic::Code::Unauthenticated => $error_id::new($code_id::Unauthenticated, status.message()),
                    tonic::Code::PermissionDenied => $error_id::new($code_id::PermissionDenied, status.message()),
                    tonic::Code::Ok => panic!("TODO shouldn't happen..."),
                    code => $error_id::new($code_id::Other(GrpcCode::from(code)), status.message()),
                }
            }
        }
        pub type $result_id<T> = Result<T, $error_id>;
    };
}
