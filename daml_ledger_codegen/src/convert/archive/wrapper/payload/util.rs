use crate::convert::error::{DamlCodeGenError, DamlCodeGenResult};

/// Required value.
pub trait Required<T> {
    fn req(self) -> DamlCodeGenResult<T>;
}

impl<T> Required<T> for Option<T> {
    fn req(self) -> DamlCodeGenResult<T> {
        self.ok_or_else(|| DamlCodeGenError::MissingRequiredField)
    }
}
