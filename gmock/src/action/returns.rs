use crate::Pointee;

use super::Action;

/* Return */

pub fn return_<T>(value: T) -> Return<T> {
    Return(value)
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Return<T>(pub T);

impl<T, X> Action<X, T> for Return<T> {
    fn exec(self, _args: X) -> T {
        self.0
    }
}

/* ReturnRef */

pub fn return_ref<T>(value: &T) -> ReturnRef<'_, T> {
    ReturnRef(value)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ReturnRef<'a, T>(pub &'a T);

impl<'a, T, X> Action<X, &'a T> for ReturnRef<'a, T> {
    fn exec(self, _args: X) -> &'a T {
        self.0
    }
}

/* ReturnPointee  */

pub fn return_pointee<T>(value: T) -> ReturnPointee<T> {
    ReturnPointee(value)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ReturnPointee<T>(pub T);

impl<P, T, X> Action<X, T> for ReturnPointee<P>
where
    P: Pointee<T>,
{
    fn exec(self, _args: X) -> T {
        self.0.get()
    }
}
