use std::borrow::Cow;

/// Convert a type containing [`Cow`] fields to have static lifetimes.
///
/// Similar to [`ToOwned`], this trait is for for converting struct `S<'a>` to `S<'static>`.
pub trait ToStatic {
    type Static: 'static;

    /// Convert to an an owned (static lifetime) instance of Self.
    fn to_static(&self) -> Self::Static;
}

/// Blanket [`ToStatic`] impl for converting `Cow<'a, T: ?Sized>` to `Cow<'static, T: ?Sized>`.
impl<T> ToStatic for Cow<'_, T>
where
    T: 'static + ToOwned + ?Sized,
{
    type Static = Cow<'static, T>;

    fn to_static(&self) -> Self::Static {
        Cow::Owned(self.clone().into_owned())
    }
}
