use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::Matcher;

pub fn any() -> Any {
    Any
}

#[must_use]
#[derive(Debug)]
pub struct Any;

impl<T> Matcher<T> for Any {
    fn matches(&self, _value: &T) -> bool {
        true
    }
}

impl Display for Any {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "any")
    }
}
