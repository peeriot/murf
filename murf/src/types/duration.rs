use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::time::Duration as StdDuration;

use parse_duration::{parse, parse::Error};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Duration(pub StdDuration);

macro_rules! impl_from {
    (fn $name:ident(value: $type:ty)) => {
        pub fn $name(value: $type) -> Self {
            Self(StdDuration::$name(value))
        }
    };
}

impl Duration {
    impl_from!(fn from_secs(value: u64));
    impl_from!(fn from_millis(value: u64));
    impl_from!(fn from_micros(value: u64));
    impl_from!(fn from_nanos(value: u64));
    impl_from!(fn from_secs_f32(value: f32));
    impl_from!(fn from_secs_f64(value: f64));
}

impl From<StdDuration> for Duration {
    fn from(value: StdDuration) -> Self {
        Self(value)
    }
}

impl From<Duration> for StdDuration {
    fn from(value: Duration) -> Self {
        value.0
    }
}

impl FromStr for Duration {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s).map(Self)
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let secs = self.0.as_secs();
        let nanos = self.0.subsec_nanos();

        write!(f, "{secs}.{nanos:09}")
    }
}

impl PartialEq<StdDuration> for Duration {
    fn eq(&self, other: &StdDuration) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<StdDuration> for Duration {
    fn partial_cmp(&self, other: &StdDuration) -> Option<Ordering> {
        Some(self.0.cmp(other))
    }
}

impl Deref for Duration {
    type Target = StdDuration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Duration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
