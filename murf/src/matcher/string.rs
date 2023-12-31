use std::fmt::{Formatter, Result as FmtResult};

use super::Matcher;

/* IsEmpty */

pub fn is_empty() -> IsEmpty {
    IsEmpty
}

pub struct IsEmpty;

impl<X> Matcher<X> for IsEmpty
where
    X: AsRef<str>,
{
    fn matches(&self, value: &X) -> bool {
        value.as_ref().is_empty()
    }

    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "IsEmpty")
    }
}

macro_rules! impl_str_matcher {
    ($type:ident, str::$method:ident, $fmt:tt) => {
        pub fn $method<P: Into<String>>(pattern: P) -> $type {
            $type(pattern.into())
        }

        pub struct $type(String);

        impl<X> Matcher<X> for $type
        where
            X: AsRef<str>,
        {
            fn matches(&self, value: &X) -> bool {
                value.as_ref().$method(&self.0)
            }

            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                write!(f, $fmt, self.0)
            }
        }
    };
}

impl_str_matcher!(StartsWith, str::starts_with, "StartsWith({})");
impl_str_matcher!(EndsWith, str::ends_with, "EndsWith({})");
impl_str_matcher!(Contains, str::contains, "Contains({})");
