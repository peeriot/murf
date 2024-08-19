use std::fmt::{Display, Formatter, Result as FmtResult};

use super::Matcher;

/// Create a new [`IsEmpty`] matcher, that matches any kind of string, that is empty.
pub fn is_empty() -> IsEmpty {
    IsEmpty
}

/// Implements a [`Matcher`] that matches any kind of string that is empty.
#[must_use]
#[derive(Debug)]
pub struct IsEmpty;

impl<X> Matcher<X> for IsEmpty
where
    X: AsRef<str>,
{
    fn matches(&self, value: &X) -> bool {
        value.as_ref().is_empty()
    }
}

impl Display for IsEmpty {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "IsEmpty")
    }
}

macro_rules! impl_str_matcher {
    ($type:ident, str::$method:ident, $fmt:tt, $ctor_doc:expr, $type_doc:expr) => {
        #[doc = $ctor_doc]
        pub fn $method<P: Into<String>>(pattern: P) -> $type {
            $type(pattern.into())
        }

        #[derive(Debug)]
        #[doc = $type_doc]
        pub struct $type(String);

        impl<X> Matcher<X> for $type
        where
            X: AsRef<str>,
        {
            fn matches(&self, value: &X) -> bool {
                value.as_ref().$method(&self.0)
            }
        }

        impl Display for $type {
            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                write!(f, $fmt, self.0)
            }
        }
    };
}

impl_str_matcher!(
    StartsWith,
    str::starts_with,
    "StartsWith({})",
    "Create a new [`StartsWith`] matcher, that matches any kind of string, that starts with the passed `pattern`.",
    "Implements a [`Matcher'] that matches any kind of string, that starts with the passed pattern."
);
impl_str_matcher!(
    EndsWith,
    str::ends_with,
    "EndsWith({})",
    "Create a new [`EndsWith`] matcher, that matches any kind of string, that ends with the passed `pattern`.",
    "Implements a [`EndsWith'] that matches any kind of string, that starts with the passed pattern."
);
impl_str_matcher!(
    Contains,
    str::contains,
    "Contains({})",
    "Create a new [`Contains`] matcher, that matches any kind of string, that contains the passed `pattern`.",
    "Implements a [`Contains'] that matches any kind of string, that starts with the passed pattern."
);
