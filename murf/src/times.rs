use std::ops::{
    Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};
use std::sync::atomic::{AtomicUsize, Ordering};

/* Times */

#[derive(Default)]
pub struct Times {
    pub count: AtomicUsize,
    pub range: TimesRange,
}

impl Times {
    pub fn new<R: Into<TimesRange>>(range: R) -> Self {
        Self {
            count: Default::default(),
            range: range.into(),
        }
    }

    pub fn increment(&self) -> usize {
        self.count.fetch_add(1, Ordering::Relaxed)
    }

    pub fn is_ready(&self) -> bool {
        match &self.range.start {
            Bound::Unbounded => true,
            Bound::Included(x) => *x <= self.count.load(Ordering::Relaxed),
            Bound::Excluded(x) => *x < self.count.load(Ordering::Relaxed),
        }
    }

    pub fn is_done(&self) -> bool {
        match &self.range.end {
            Bound::Unbounded => false,
            Bound::Included(x) => self.count.load(Ordering::Relaxed) >= *x,
            Bound::Excluded(x) => self.count.load(Ordering::Relaxed) + 1 >= *x,
        }
    }
}

/* TimesRange */

pub struct TimesRange {
    start: Bound<usize>,
    end: Bound<usize>,
}

impl Default for TimesRange {
    fn default() -> Self {
        Self {
            start: Bound::Unbounded,
            end: Bound::Unbounded,
        }
    }
}

impl From<usize> for TimesRange {
    fn from(value: usize) -> Self {
        Self {
            start: Bound::Included(value),
            end: Bound::Included(value),
        }
    }
}

macro_rules! impl_from_range_bounds {
    ($x:ty) => {
        impl From<$x> for TimesRange {
            fn from(value: $x) -> Self {
                Self {
                    start: value.start_bound().cloned(),
                    end: value.end_bound().cloned(),
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
        assert_eq!(false, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(true, t.is_done());
    }

    #[test]
    fn range() {
        let t = Times::new(1..3);
        assert_eq!(false, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(true, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(true, t.is_done());
    }

    #[test]
    fn range_from() {
        let t = Times::new(2..);
        assert_eq!(false, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(false, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
    }

    #[test]
    fn range_full() {
        let t = Times::new(..);
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
    }

    #[test]
    fn range_inclusive() {
        let t = Times::new(2..=3);
        assert_eq!(false, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(false, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(true, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(true, t.is_done());
    }

    #[test]
    fn range_to() {
        let t = Times::new(..3);
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(true, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(true, t.is_done());
    }

    #[test]
    fn range_to_inclusive() {
        let t = Times::new(..=3);
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(false, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(true, t.is_done());
        t.increment();
        assert_eq!(true, t.is_ready());
        assert_eq!(true, t.is_done());
    }
}
