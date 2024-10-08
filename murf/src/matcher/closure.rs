use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::Matcher;

/// Create a new [`Closure`] matcher that executes the passed function `f` to
/// verify if a argument matches the expectation.
pub fn closure<F>(f: F) -> Closure<F> {
    Closure(f)
}

/// Implements a [`Matcher`] that executes the passed function `F` to
/// verify if a argument matches the expectation.
#[must_use]
#[derive(Debug)]
pub struct Closure<F>(pub F);

impl<T, F> Matcher<T> for Closure<F>
where
    F: Fn(&T) -> bool,
{
    fn matches(&self, value: &T) -> bool {
        self.0(value)
    }
}

impl<F> Display for Closure<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Closure")
    }
}
