use crate::Pointee;

use super::Action;

/// Creates a [`Return`] action that returns the passed `value` when called.
pub fn return_<T>(value: T) -> Return<T> {
    Return(value)
}

/// Action that returns the passed value `T` when called.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Return<T>(pub T);

impl<T, X> Action<X, T> for Return<T> {
    fn exec(self, _args: X) -> T {
        self.0
    }
}

/// Create a [`ReturnRef`] action that returns a reference to the passed `value`
/// when called.
pub fn return_ref<T>(value: &T) -> ReturnRef<'_, T> {
    ReturnRef(value)
}

/// Action that returns the passed reference `&T` when called.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ReturnRef<'a, T>(pub &'a T);

impl<'a, T, X> Action<X, &'a T> for ReturnRef<'a, T> {
    fn exec(self, _args: X) -> &'a T {
        self.0
    }
}

/// Creates a [`ReturnPointee`] action that returns the value the [`Pointee`]
/// points to when called.
pub fn return_pointee<T>(value: T) -> ReturnPointee<T> {
    ReturnPointee(value)
}

/// Action that returns the value the [`Pointee`] points to when called.
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
