use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::Matcher;

pub fn closure<F>(f: F) -> Closure<F> {
    Closure(f)
}

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
