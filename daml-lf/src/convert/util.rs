use crate::error::{DamlLfConvertError, DamlLfConvertResult};

/// Required value.
pub trait Required<T> {
    fn req(self) -> DamlLfConvertResult<T>;
}

impl<T> Required<T> for Option<T> {
    fn req(self) -> DamlLfConvertResult<T> {
        self.ok_or(DamlLfConvertError::MissingRequiredField)
    }
}
