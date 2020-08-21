use thiserror::Error;

#[derive(Error, Debug)]
pub enum DarnError {
    #[error("unknown package {0}")]
    UnknownPackage(String),
    #[error("unknown module {0} in package {1}")]
    UnknownModule(String, String),
    #[error("error {0}")]
    Other(String),
}

impl DarnError {
    pub fn unknown_package(package: &str) -> Self {
        DarnError::UnknownPackage(package.to_owned())
    }

    pub fn unknown_module(module: &str, package: &str) -> Self {
        DarnError::UnknownModule(module.to_owned(), package.to_owned())
    }

    pub fn other_error(msg: impl Into<String>) -> Self {
        DarnError::Other(msg.into())
    }
}
