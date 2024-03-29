use daml_lf::DamlLfError;
use std::fmt;
use std::fmt::{Display, Formatter};

/// Daml code generator errors.
#[derive(Debug)]
pub enum DamlCodeGenError {
    /// AN invalid module matcher regex was provided.
    InvalidModuleMatcherRegex(regex::Error),
    /// Daml LF error.
    DamlLfError(DamlLfError),
    /// IO error.
    IoError(std::io::Error),
}

impl std::error::Error for DamlCodeGenError {}
impl Display for DamlCodeGenError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DamlCodeGenError::InvalidModuleMatcherRegex(e) =>
                write!(fmt, "InvalidModuleMatcherRegex {}", (e as &regex::Error)),
            DamlCodeGenError::DamlLfError(e) => write!(fmt, "Daml LF error {}", (e as &DamlLfError)),
            DamlCodeGenError::IoError(e) => write!(fmt, "IOError: {}", (e as &std::io::Error)),
        }
    }
}

/// Daml code generator result.
pub type DamlCodeGenResult<T> = Result<T, DamlCodeGenError>;
