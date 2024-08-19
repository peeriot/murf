mod invoke;
mod returns;

pub use invoke::{invoke, Invoke};
pub use returns::{return_, return_pointee, return_ref, Return, ReturnPointee, ReturnRef};

/* Action */

pub trait Action<T, R> {
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

/* RepeatableAction */

pub trait RepeatableAction<T, R> {
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

/* OnetimeAction */

#[derive(Debug)]
pub struct OnetimeAction<X>(Option<X>);

impl<X> OnetimeAction<X> {
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

/* RepeatedAction */

#[derive(Debug)]
pub struct RepeatedAction<X>(X);

impl<X> RepeatedAction<X> {
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
