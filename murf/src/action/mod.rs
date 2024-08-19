//! The [`action`](self) module contains difference pre-defined actions that may
//! be executed for a call-expectation of a mocked type.

mod invoke;
mod returns;

pub use invoke::{invoke, Invoke};
pub use returns::{return_, return_pointee, return_ref, Return, ReturnPointee, ReturnRef};

/// Trait that defines an action that can only be executed once.
///
/// This is similar to [`FnOnce`] of the standard library.
///
/// The arguments passed to the action are either
/// - a unit `()` for no arguments
/// - a single type `T` for one argument
/// - or a tuple `(T1, T2, ...)` of many arguments
pub trait Action<T, R> {
    /// Execute the action with the passed arguments.
    fn exec(self, args: T) -> R;
}

impl<X, T, R> Action<T, R> for X
where
    X: FnOnce(T) -> R,
{
    fn exec(self, args: T) -> R {
        self(args)
    }
}

/// Like [`Action`] but this action may be called repeatedly.
///
/// This is similar to [`FnMut`] of the standard library.
pub trait RepeatableAction<T, R> {
    /// Execute the action with the passed arguments.
    fn exec(&mut self, args: T) -> R;
}

impl<X, T, R> RepeatableAction<T, R> for X
where
    X: FnMut(T) -> R,
{
    fn exec(&mut self, args: T) -> R {
        self(args)
    }
}

/// Helper type to implement [`RepeatableAction`] for a action that can only be
/// called once. Any further call will panic!
#[derive(Debug)]
pub struct OnetimeAction<X>(Option<X>);

impl<X> OnetimeAction<X> {
    /// Create a new [`OnetimeAction`] instance.
    pub fn new(inner: X) -> Self {
        Self(Some(inner))
    }
}

impl<T, R, X> RepeatableAction<T, R> for OnetimeAction<X>
where
    X: Action<T, R>,
{
    fn exec(&mut self, args: T) -> R {
        self.0
            .take()
            .expect("Action was already executed")
            .exec(args)
    }
}

/// Helper type to implement [`RepeatableAction`] for any action that implements
/// [`Action`] and [`Clone`].
#[derive(Debug)]
pub struct RepeatedAction<X>(X);

impl<X> RepeatedAction<X> {
    /// Create a new [`RepeatedAction`] instance.
    pub fn new(inner: X) -> Self {
        Self(inner)
    }
}

impl<X, T, R> RepeatableAction<T, R> for RepeatedAction<X>
where
    X: Action<T, R> + Clone,
{
    fn exec(&mut self, args: T) -> R {
        self.0.clone().exec(args)
    }
}
