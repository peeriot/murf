use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};

/* Times */

#[derive(Default)]
pub struct Times {
    pub count: AtomicUsize,
    pub range: TimesRange,
}

impl Times {
    pub fn increment(&self) -> usize {
        self.count.fetch_add(1, Ordering::Relaxed)
    }

    pub fn is_ready(&self) -> bool {
        self.count.load(Ordering::Relaxed) >= self.range.0.start
    }

    pub fn is_done(&self) -> bool {
        self.count.load(Ordering::Relaxed) >= self.range.0.end
    }
}

/* TimesRange */

pub struct TimesRange(pub Range<usize>);

impl Default for TimesRange {
    fn default() -> Self {
        Self(0..usize::max_value())
    }
}

impl From<usize> for TimesRange {
    fn from(value: usize) -> Self {
        Self(value..(value + 1))
    }
}

impl From<Range<usize>> for TimesRange {
    fn from(value: Range<usize>) -> Self {
        Self(value)
    }
}
