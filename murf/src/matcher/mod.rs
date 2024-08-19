mod any;
mod closure;
mod compare;
mod deref;
mod inspect;
mod multi;
mod no_args;
mod range;
mod string;

use std::fmt::{Formatter, Result as FmtResult};

pub use any::{any, Any};
pub use closure::{closure, Closure};
pub use compare::{eq, ge, gt, le, lt, ne, Eq, Ge, Gt, Le, Lt, Ne};
pub use deref::{deref, Deref};
pub use inspect::{inspect, Inspect};

pub use multi::{multi, Multi};
pub use no_args::{no_args, NoArgs};
pub use range::{range, Range};
pub use string::{
    contains as str_contains, ends_with as str_ends_with, is_empty, starts_with as str_starts_with,
    Contains as StrContains, EndsWith as StrEndsWith, IsEmpty, StartsWith as StrStartsWith,
};

/// A matcher is used to check if the passed argument matches a pre-defined
/// expectation. It is mostly used to verify the arguments to an expected call.
pub trait Matcher<T> {
    /// Returns `true` if the passed `value` matches the expectations, `false`
    /// otherwise.
    fn matches(&self, value: &T) -> bool;

    /// Write a human readable representation of the matcher to the passed
    /// formatter.
    ///
    /// # Errors
    /// Returns an error if writing to the formatter failed.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult;
}
