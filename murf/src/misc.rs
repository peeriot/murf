//! The [`misc`](self) crate contains different helper types and traits.

use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

/// Helper type that is used to the values a pointer like type is pointing to.
///
/// This is mostly used in the [`ReturnPointee`](crate::action::ReturnPointee) type.
pub trait Pointee<T> {
    /// Get the value the pointer type is currently pointing at.
    fn get(&self) -> T;
}

impl<T> Pointee<T> for Rc<RefCell<T>>
where
    T: Clone,
{
    fn get(&self) -> T {
        RefCell::borrow(self).clone()
    }
}

impl<T> Pointee<T> for Arc<Mutex<T>>
where
    T: Clone,
{
    fn get(&self) -> T {
        self.lock().unwrap().clone()
    }
}

/// Implements [`Pointee`] for any type `T` that implements [`Borrow`].
#[derive(Debug)]
pub struct Borrowed<T>(T);

impl<T, X> Pointee<T> for Borrowed<X>
where
    X: Borrow<T>,
    T: Clone,
{
    fn get(&self) -> T {
        self.0.borrow().clone()
    }
}

/// Implements [`Pointee`] for any pointer type `* const T`.
#[derive(Debug)]
pub struct Pointer<T>(pub *const T);

impl<T> Pointee<T> for Pointer<T>
where
    T: Clone,
{
    fn get(&self) -> T {
        unsafe { &*self.0 }.clone()
    }
}

/// Defines a expectation for a function call on a mocked object.
pub trait Expectation: Display {
    /// Returns the type id of the expectation.
    fn type_id(&self) -> usize;

    /// Returns `true` if this expectation is ready, `false` otherwise.
    ///
    /// Ready means that the expectation was executed the expected amount of times.
    fn is_ready(&self) -> bool;

    /// Mark this expectation as done.
    ///
    /// Done means that this expectation has been finished and will not called again.
    fn set_done(&self);

    /// Get the type signature of the expectation.
    fn type_signature(&self) -> &'static str;
}

/// Get the next type id
pub fn next_type_id() -> usize {
    NEXT_TYPE_ID.fetch_add(1, Ordering::Relaxed)
}

static NEXT_TYPE_ID: AtomicUsize = AtomicUsize::new(0);
