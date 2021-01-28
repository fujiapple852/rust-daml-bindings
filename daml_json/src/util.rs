use crate::error::{DamlJsonCodecError, DamlJsonCodecResult};

/// Required value.
pub trait Required<T> {
    fn req(self) -> DamlJsonCodecResult<T>;
}

impl<T> Required<T> for Option<T> {
    fn req(self) -> DamlJsonCodecResult<T> {
        self.ok_or(DamlJsonCodecError::MissingRequiredField)
    }
}

/// Interpret a Vec as a slice of exactly 1 entry or an error.
pub trait AsSingleSliceExt<T> {
    fn as_single(&self) -> DamlJsonCodecResult<&T>;
}

impl<T> AsSingleSliceExt<T> for Vec<T> {
    fn as_single(&self) -> DamlJsonCodecResult<&T> {
        match self.as_slice() {
            [single] => Ok(single),
            _ => Err(DamlJsonCodecError::UnexpectedListEntries),
        }
    }
}

/// Return A from a tuple of type (A, B)
pub fn fst<A, B>((a, _): (A, B)) -> A {
    a
}
