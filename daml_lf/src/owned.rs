use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

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

/// Blanket [`ToStatic`] impl for `Vec<T>`.
impl<T> ToStatic for Vec<T>
where
    T: ToStatic,
{
    type Static = Vec<T::Static>;

    fn to_static(&self) -> Self::Static {
        self.iter().map(ToStatic::to_static).collect()
    }
}

/// Blanket [`ToStatic`] impl for `HashMap<K, V>`.
impl<K, V, S: BuildHasher> ToStatic for HashMap<K, V, S>
where
    K: ToStatic,
    K::Static: Eq + Hash,
    V: ToStatic,
{
    type Static = HashMap<K::Static, V::Static>;

    fn to_static(&self) -> Self::Static {
        self.iter().map(|(k, v)| (k.to_static(), v.to_static())).collect()
    }
}

/// Blanket [`ToStatic`] impl for `Option<T>`.
impl<T> ToStatic for Option<T>
where
    T: ToStatic,
{
    type Static = Option<T::Static>;

    fn to_static(&self) -> Self::Static {
        self.as_ref().map(ToStatic::to_static)
    }
}

/// Blanket [`ToStatic`] impl for `Box<T>`.
impl<T> ToStatic for Box<T>
where
    T: ToStatic,
{
    type Static = Box<T::Static>;

    fn to_static(&self) -> Self::Static {
        Box::new(self.as_ref().to_static())
    }
}
