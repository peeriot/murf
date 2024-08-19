//! The [`times`](self) module contains different types and helpers to define
//! how often a call expectation may be called.

use std::ops::{
    Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Type to keep track of the number of calls expected for a specific call expectation.
#[derive(Default, Debug)]
pub struct Times {
    /// Number of calls the expectation was already executed.
    pub count: AtomicUsize,

    /// Expected number of calls.
    pub range: TimesRange,
}

impl Times {
    /// Create a new [`Times`] instance from the passed `range`.
    pub fn new<R: Into<TimesRange>>(range: R) -> Self {
        Self {
            count: AtomicUsize::default(),
            range: range.into(),
        }
    }

    /// Increment the current call count.
    pub fn increment(&self) -> usize {
        self.count.fetch_add(1, Ordering::Relaxed)
    }

    /// Return `true` if lower bound of the range is fulfilled.
    pub fn is_ready(&self) -> bool {
        match &self.range.lower {
            Bound::Unbounded => true,
            Bound::Included(x) => *x <= self.count.load(Ordering::Relaxed),
            Bound::Excluded(x) => *x < self.count.load(Ordering::Relaxed),
        }
    }

    /// Return `true` if upper bound of the range is fulfilled.
    pub fn is_done(&self) -> bool {
        match &self.range.upper {
            Bound::Unbounded => false,
            Bound::Included(x) => self.count.load(Ordering::Relaxed) >= *x,
            Bound::Excluded(x) => self.count.load(Ordering::Relaxed) + 1 >= *x,
        }
    }
}

/// Defines the range of expected calls with a lower and a upper limit.
///
/// Similar to [`RangeBounds`] from the standard library but as struct instead
/// of trait.
#[derive(Debug)]
pub struct TimesRange {
    lower: Bound<usize>,
    upper: Bound<usize>,
}

impl Default for TimesRange {
    fn default() -> Self {
        Self {
            lower: Bound::Unbounded,
            upper: Bound::Unbounded,
        }
    }
}

impl From<usize> for TimesRange {
    fn from(value: usize) -> Self {
        Self {
            lower: Bound::Included(value),
            upper: Bound::Included(value),
        }
    }
}

macro_rules! impl_from_range_bounds {
    ($x:ty) => {
        impl From<$x> for TimesRange {
            fn from(value: $x) -> Self {
                Self {
                    lower: value.start_bound().cloned(),
                    upper: value.end_bound().cloned(),
                }
            }
        }
    };
}

impl_from_range_bounds!(Range<usize>);
impl_from_range_bounds!(RangeFrom<usize>);
impl_from_range_bounds!(RangeFull);
impl_from_range_bounds!(RangeInclusive<usize>);
impl_from_range_bounds!(RangeTo<usize>);
impl_from_range_bounds!(RangeToInclusive<usize>);

#[cfg(test)]
mod tests {
    use super::Times;

    #[test]
    fn number() {
        let t = Times::new(1);
        assert!(!t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(t.is_done());
    }

    #[test]
    fn range() {
        let t = Times::new(1..3);
        assert!(!t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(t.is_done());
    }

    #[test]
    fn range_from() {
        let t = Times::new(2..);
        assert!(!t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(!t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(!t.is_done());
    }

    #[test]
    fn range_full() {
        let t = Times::new(..);
        assert!(t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(!t.is_done());
    }

    #[test]
    fn range_inclusive() {
        let t = Times::new(2..=3);
        assert!(!t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(!t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(t.is_done());
    }

    #[test]
    fn range_to() {
        let t = Times::new(..3);
        assert!(t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(t.is_done());
    }

    #[test]
    fn range_to_inclusive() {
        let t = Times::new(..=3);
        assert!(t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(t.is_done());
    }

    #[test]
    fn range_zero_to_one() {
        let t = Times::new(0..=1);
        assert!(t.is_ready());
        assert!(!t.is_done());
        t.increment();
        assert!(t.is_ready());
        assert!(t.is_done());
    }
}
