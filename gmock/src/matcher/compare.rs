use std::fmt::{Display, Formatter, Result as FmtResult};

use super::Matcher;

macro_rules! impl_matcher {
    ($type:ident, $trait:ident::$method:ident, $fmt:tt) => {
        pub fn $method<T>(value: T) -> $type<T> {
            $type(value)
        }

        pub struct $type<T>(pub T);

        impl<T, X> Matcher<X> for $type<T>
        where
            T: $trait<X> + Display,
        {
            fn matches(&self, value: &X) -> bool {
                self.0.$method(value)
            }

            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                write!(f, $fmt, self.0)
            }
        }
    };
}

impl_matcher!(Eq, PartialEq::eq, "Eq({})");
impl_matcher!(Ne, PartialEq::ne, "Ne({})");

impl_matcher!(Lt, PartialOrd::lt, "Lt({})");
impl_matcher!(Le, PartialOrd::le, "Le({})");
impl_matcher!(Gt, PartialOrd::gt, "Gt({})");
impl_matcher!(Ge, PartialOrd::ge, "Ge({})");
