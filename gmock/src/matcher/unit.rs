use std::fmt::{Formatter, Result as FmtResult};

use crate::Matcher;

pub fn unit() -> Unit {
    Unit
}

pub struct Unit;

impl Matcher<()> for Unit {
    fn matches(&self, _value: &()) -> bool {
        true
    }

    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "unit")
    }
}
