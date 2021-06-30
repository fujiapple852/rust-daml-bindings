use itertools::Itertools;
use thiserror::Error;

use daml::lf::element::DamlModule;

#[derive(Error, Debug)]
#[error("required value was not supplied")]
pub struct RequiredError;

/// Required value.
pub trait Required<T> {
    fn req(self) -> Result<T, RequiredError>;
}

impl<T> Required<T> for Option<T> {
    fn req(self) -> Result<T, RequiredError> {
        self.ok_or(RequiredError)
    }
}

#[derive(Error, Debug)]
#[error("unknown module path {0}")]
pub struct UnknownModulePathError(String);

pub trait ChildModulePathOrError {
    fn child_module_path_or_err<S: AsRef<str>>(
        &self,
        relative_path: &[S],
    ) -> Result<&DamlModule<'_>, UnknownModulePathError>;
}

impl ChildModulePathOrError for DamlModule<'_> {
    fn child_module_path_or_err<S: AsRef<str>>(
        &self,
        relative_path: &[S],
    ) -> Result<&DamlModule<'_>, UnknownModulePathError> {
        self.child_module_path(relative_path)
            .ok_or_else(|| UnknownModulePathError(relative_path.iter().map(AsRef::as_ref).join(".")))
    }
}
