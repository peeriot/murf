use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::Matcher;

/// Create a new [`Deref`] matcher, that calls the [`deref`](std::ops::Deref::deref())
/// method of the argument and forwards it to the passed `inner` matcher.
pub fn deref<M>(inner: M) -> Deref<M> {
    Deref(inner)
}

/// Implements a [`Matcher`] that calls the [`deref`](std::ops::Deref::deref())
/// method of the argument and forwards it to the passed matcher `M`.
#[must_use]
#[derive(Debug)]
pub struct Deref<M>(pub M);

impl<T, M> Matcher<T> for Deref<M>
where
    T: std::ops::Deref,
    T::Target: Sized,
    M: Matcher<T::Target>,
{
    fn matches(&self, value: &T) -> bool {
        self.0.matches(&**value)
    }
}

impl<M> Display for Deref<M>
where
    M: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "deref(")?;
        self.0.fmt(f)?;
        write!(f, ")")?;

        Ok(())
    }
}
