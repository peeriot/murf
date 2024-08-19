use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use super::Matcher;

macro_rules! impl_matcher {
    ($type:ident, $trait:ident::$method:ident, $fmt:tt, $ctor_doc:expr, $type_doc:expr) => {
        #[doc = $ctor_doc]
        pub fn $method<T>(value: T) -> $type<T> {
            $type(value)
        }

        #[must_use]
        #[derive(Debug)]
        #[doc = $type_doc]
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

impl_matcher!(
    Eq,
    PartialEq::eq,
    "Eq({:?})",
    "Create a new [`Eq`](Eq) matcher that checks if a argument is equal to the passed `value`.",
    "Implements a [`Matcher`] that checks if a argument is equal to the passed value `T`."
);
impl_matcher!(
    Ne,
    PartialEq::ne,
    "Ne({:?})",
    "Create a new [`Ne`] matcher that checks if a argument is not equal to the passed `value`.",
    "Implements a [`Matcher`] that checks if a argument is not equal to the passed value `T`."
);
impl_matcher!(
    Lt,
    PartialOrd::lt,
    "Lt({:?})",
    "Create a new [`Lt`] matcher that checks if a argument is lower than the passed `value`.",
    "Implements a [`Matcher`] that checks if a argument is lower than the passed value `T`."
);
impl_matcher!(
    Le,
    PartialOrd::le,
    "Le({:?})",
    "Create a new [`Le`] matcher that checks if a argument is lower or equal to the passed `value`.",
    "Implements a [`Matcher`] that checks if a argument is lower or equal to the passed value `T`."
);
impl_matcher!(
    Gt,
    PartialOrd::gt,
    "Gt({:?})",
    "Create a new [`Gt`] matcher that checks if a argument is greater than the passed `value`.",
    "Implements a [`Matcher`] that checks if a argument is greater than the passed value `T`."
);
impl_matcher!(
    Ge,
    PartialOrd::ge,
    "Ge({:?})", "Create a new [`Ge`] matcher that checks if a argument is greater or equal to the passed `value`.",
    "Implements a [`Matcher`] that checks if a argument is greater or equal to the passed value `T`."
);
