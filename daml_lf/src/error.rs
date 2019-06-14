use crate::error::DamlLfError::{DarParseError, IOError};
use std::error;
use std::fmt;
use zip::result::ZipError;

/// Represents `DAML-LF` specific errors.
#[derive(Debug)]
pub enum DamlLfError {
    DamlLfParseError(prost::DecodeError),
    DarParseError(String),
    StdError(Box<dyn std::error::Error>),
    IOError(std::io::Error),
    UnknownVersion,
    UnsupportedVersion,
    Other(String),
}

impl DamlLfError {
    pub fn new_dar_parse_error(error: impl Into<String>) -> Self {
        DamlLfError::DarParseError(error.into())
    }
}

impl fmt::Display for DamlLfError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DamlLfError::Other(e) => write!(fmt, "Other: {}", e),
            DamlLfError::DamlLfParseError(e) => write!(fmt, "ParseError: {}", (e as &prost::DecodeError)),
            DamlLfError::DarParseError(e) => write!(fmt, "DarParseError: {}", e),
            DamlLfError::IOError(e) => write!(fmt, "IOError: {}", (e as &std::io::Error)),
            DamlLfError::StdError(e) => write!(fmt, "StdError: {}", (e as &Box<dyn std::error::Error>)),
            DamlLfError::UnknownVersion => write!(fmt, "unknown DAML-LF version"),
            DamlLfError::UnsupportedVersion => write!(fmt, "unsupported DAML-LF version"),
        }
    }
}

impl error::Error for DamlLfError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            DamlLfError::DamlLfParseError(e) => Some(e),
            DamlLfError::IOError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<prost::DecodeError> for DamlLfError {
    fn from(e: prost::DecodeError) -> Self {
        DamlLfError::DamlLfParseError(e)
    }
}

impl From<&str> for DamlLfError {
    fn from(e: &str) -> Self {
        DamlLfError::Other(e.to_owned())
    }
}

impl From<Box<dyn std::error::Error>> for DamlLfError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        DamlLfError::StdError(e)
    }
}

impl From<std::io::Error> for DamlLfError {
    fn from(e: std::io::Error) -> Self {
        IOError(e)
    }
}

impl From<yaml_rust::scanner::ScanError> for DamlLfError {
    fn from(e: yaml_rust::scanner::ScanError) -> Self {
        DarParseError(e.to_string())
    }
}

impl From<ZipError> for DamlLfError {
    fn from(e: ZipError) -> Self {
        DarParseError(e.to_string())
    }
}

pub type DamlLfResult<T> = ::std::result::Result<T, DamlLfError>;
