use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use super::Matcher;

macro_rules! impl_matcher {
    ($type:ident, $trait:ident::$method:ident, $fmt:tt) => {
        pub fn $method<T>(value: T) -> $type<T> {
            $type(value)
        }

        #[must_use]
        #[derive(Debug)]
        pub struct $type<T>(pub T);

        impl<T, X> Matcher<X> for $type<T>
        where
            T: $trait<X> + Debug,
        {
            fn matches(&self, value: &X) -> bool {
                self.0.$method(value)
            }
        }

        impl<T> Display for $type<T>
        where
            T: Debug,
        {
            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                write!(f, $fmt, self.0)
            }
        }
    };
}

impl_matcher!(Eq, PartialEq::eq, "Eq({:?})");
impl_matcher!(Ne, PartialEq::ne, "Ne({:?})");

impl_matcher!(Lt, PartialOrd::lt, "Lt({:?})");
impl_matcher!(Le, PartialOrd::le, "Le({:?})");
impl_matcher!(Gt, PartialOrd::gt, "Gt({:?})");
impl_matcher!(Ge, PartialOrd::ge, "Ge({:?})");
