use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::Matcher;

/// Creates a new [`NoArgs`] matcher, that matches only the empty parameter list
/// `()` and always returns `true`.
pub fn no_args() -> NoArgs {
    NoArgs
}

/// Implements a [`Matcher`], that matches only the empty parameter list `()` and
/// always returns `true`.
#[must_use]
#[derive(Debug)]
pub struct NoArgs;

impl Matcher<()> for NoArgs {
    fn matches(&self, _value: &()) -> bool {
        true
    }
}

impl Display for NoArgs {
    fn fmt(&self, _: &mut Formatter<'_>) -> FmtResult {
        Ok(())
    }
}
