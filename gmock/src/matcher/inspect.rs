use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use crate::Matcher;

pub fn inspect<M>(inner: M) -> Inspect<M> {
    Inspect(inner)
}

pub struct Inspect<M>(pub M);

impl<T, M> Matcher<T> for Inspect<M>
where
    T: Debug,
    M: Matcher<T>,
{
    fn matches(&self, value: &T) -> bool {
        println!(
            "{}",
            FormatHelper {
                matcher: &self.0,
                value
            }
        );

        self.0.matches(value)
    }

    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

struct FormatHelper<'a, M, T> {
    matcher: &'a M,
    value: &'a T,
}

impl<'a, M, T> Display for FormatHelper<'a, M, T>
where
    M: Matcher<T>,
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Expect ")?;
        self.matcher.fmt(f)?;
        write!(f, " to match {:?}", self.value)?;

        Ok(())
    }
}
