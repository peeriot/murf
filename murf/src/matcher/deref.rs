use std::fmt::{Formatter, Result as FmtResult};

use crate::Matcher;

pub fn deref<M>(inner: M) -> Deref<M> {
    Deref(inner)
}

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

    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "deref(")?;
        self.0.fmt(f)?;
        write!(f, ")")?;

        Ok(())
    }
}
