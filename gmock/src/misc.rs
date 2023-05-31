/* Pointee */

use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub trait Pointee<T> {
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

/* Borrow */

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

/* Pointer */

pub struct Pointer<T>(pub *const T);

impl<T> Pointee<T> for Pointer<T>
where
    T: Clone,
{
    fn get(&self) -> T {
        unsafe { &*self.0 }.clone()
    }
}

/* Expectation */

pub trait Expectation: Display {
    fn is_ready(&self) -> bool;
    fn set_done(&self);
    fn args_type_id(&self) -> &'static str;
}
