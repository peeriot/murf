mod any;
mod compare;
mod multi;
mod range;
mod string;

use std::fmt::{Formatter, Result as FmtResult};

pub use any::{any, Any};
pub use compare::{eq, ge, gt, le, lt, ne, Eq, Ge, Gt, Le, Lt, Ne};
pub use multi::{multi, Multi};
pub use range::{range, Range};
pub use string::{
    contains as str_contains, ends_with as str_ends_with, is_empty, starts_with as str_starts_with,
    Contains as StrContains, EndsWith as StrEndsWith, IsEmpty, StartsWith as StrStartsWith,
};

pub trait Matcher<T> {
    fn matches(&self, value: &T) -> bool;
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult;
}