use std::fmt::{Display, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};

use crate::Matcher;

/// Creates a new [`Range`] matcher that matches if the argument it is in the passed
/// `range`.
pub fn range<R, T>(range: R) -> Range<R, T>
where
    R: RangeBounds<T>,
{
    Range::new(range)
}

/// Implements a [`Matcher`], that matches if the argument is in the passed range `R`.
#[must_use]
#[derive(Debug)]
pub struct Range<R, T> {
    range: R,
    _marker: PhantomData<T>,
}

impl<R, T> Range<R, T> {
    /// Create a new [`Range`] matcher instance.
    pub fn new(range: R) -> Self {
        Self {
            range,
            _marker: PhantomData,
        }
    }
}

impl<U, R, T> Matcher<U> for Range<R, T>
where
    R: RangeBounds<T>,
    T: PartialOrd<U> + Display,
    U: PartialOrd<T>,
{
    fn matches(&self, value: &U) -> bool {
        self.range.contains(value)
    }
}

impl<R, T> Display for Range<R, T>
where
    R: RangeBounds<T>,
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.range.start_bound() {
            Bound::Unbounded => write!(f, "[_, "),
            Bound::Included(x) => write!(f, "[{x}, "),
            Bound::Excluded(x) => write!(f, "({x}, "),
        }?;

        match self.range.end_bound() {
            Bound::Unbounded => write!(f, "_]"),
            Bound::Included(x) => write!(f, "{x}]"),
            Bound::Excluded(x) => write!(f, "{x})"),
        }?;

        Ok(())
    }
}
