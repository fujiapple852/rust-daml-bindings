use thiserror::Error;

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
#[error("expected a single item")]
pub struct NotSingleError;

/// Interpret a Vec as a slice of exactly 1 entry or an error.
pub trait AsSingleSliceExt<T> {
    fn as_single(&self) -> Result<&T, NotSingleError>;
}

impl<T> AsSingleSliceExt<T> for Vec<T> {
    fn as_single(&self) -> Result<&T, NotSingleError> {
        match self.as_slice() {
            [single] => Ok(single),
            _ => Err(NotSingleError),
        }
    }
}

/// Return A from a tuple of type (A, B)
pub fn fst<A, B>((a, _): (A, B)) -> A {
    a
}
