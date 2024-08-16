use std::fmt::{Formatter, Result as FmtResult};

use crate::Matcher;

pub fn closure<F>(f: F) -> Closure<F> {
    Closure(f)
}

pub struct Closure<F>(pub F);

impl<T, F> Matcher<T> for Closure<F>
where
    F: Fn(&T) -> bool,
{
    fn matches(&self, value: &T) -> bool {
        self.0(value)
    }

    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Closure")
    }
}
