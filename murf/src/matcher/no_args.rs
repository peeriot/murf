use std::fmt::{Formatter, Result as FmtResult};

use crate::Matcher;

pub fn no_args() -> NoArgs {
    NoArgs
}

pub struct NoArgs;

impl Matcher<()> for NoArgs {
    fn matches(&self, _value: &()) -> bool {
        true
    }

    fn fmt(&self, _: &mut Formatter<'_>) -> FmtResult {
        Ok(())
    }
}
