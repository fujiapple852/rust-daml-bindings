use daml_lf::DamlLfError;
use std::fmt;
use std::fmt::{Display, Formatter};

/// DAML code generator errors.
#[derive(Debug)]
pub enum DamlCodeGenError {
    /// A `DamlTypePayload::ContractId` contained more than one type argument.
    UnexpectedContractIdTypeArguments,
    /// A required optional field was None.
    MissingRequiredField,
    /// A DAML type not supported by the code generator was found.
    UnsupportedType(String),
    /// A DAML choice was not a `DamlDataWrapper::Record`.
    UnexpectedChoiceData,
    /// A feature supported by this archive version was not used.
    SupportedFeatureUnused(String, String, String),
    /// A feature not supported by this archive version was used.
    UnsupportedFeatureUsed(String, String, String),
    /// An unexpected `DamlDataPayload` variant was found.
    UnexpectedData,
    /// Expected a given `DamlTypePayload` but found a different `DamlTypePayload`
    UnexpectedType(String, String),
    /// Failed to lookup a `DamlPackagePayload` by id.
    UnknownPackage(String),
    /// Failed to lookup a `DamlModulePayload` by id.
    UnknownModule(String),
    /// Failed to lookup a `DamlDataPayload` by id.
    UnknownData(String),
    /// AN invalid module matcher regex was provided.
    InvalidModuleMatcherRegex(regex::Error),
    /// DAML LF error.
    DamlLfError(DamlLfError),
    /// IO error.
    IOError(std::io::Error),
}

impl std::error::Error for DamlCodeGenError {}
impl Display for DamlCodeGenError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DamlCodeGenError::UnexpectedContractIdTypeArguments => write!(fmt, "UnexpectedContractIdTypeArguments"),
            DamlCodeGenError::MissingRequiredField => write!(fmt, "MissingRequiredField"),
            DamlCodeGenError::UnexpectedType(expected, actual) =>
                write!(fmt, "UnsupportedType: expected {}, actual {}", expected, actual),
            DamlCodeGenError::UnexpectedChoiceData => write!(fmt, "UnexpectedChoiceType"),
            DamlCodeGenError::SupportedFeatureUnused(curr_ver, feature_ver, min_ver) => write!(
                fmt,
                "DAML LF version {} supports feature {} but was not used (supported as of version {})",
                curr_ver, feature_ver, min_ver
            ),
            DamlCodeGenError::UnsupportedFeatureUsed(curr_ver, feature_ver, min_ver) => write!(
                fmt,
                "DAML LF version {} does not support feature {} (requires version {})",
                curr_ver, feature_ver, min_ver
            ),
            DamlCodeGenError::UnexpectedData => write!(fmt, "UnexpectedData"),
            DamlCodeGenError::UnsupportedType(s) => write!(fmt, "UnexpectedType {}", s),
            DamlCodeGenError::UnknownPackage(s) => write!(fmt, "Unknown package {}", s),
            DamlCodeGenError::UnknownModule(s) => write!(fmt, "Unknown module {}", s),
            DamlCodeGenError::UnknownData(s) => write!(fmt, "Unknown data {}", s),
            DamlCodeGenError::InvalidModuleMatcherRegex(e) =>
                write!(fmt, "InvalidModuleMatcherRegex {}", (e as &regex::Error)),
            DamlCodeGenError::DamlLfError(e) => write!(fmt, "DAML LF error {}", (e as &DamlLfError)),
            DamlCodeGenError::IOError(e) => write!(fmt, "IOError: {}", (e as &std::io::Error)),
        }
    }
}

/// DAML code generator result.
pub type DamlCodeGenResult<T> = Result<T, DamlCodeGenError>;
