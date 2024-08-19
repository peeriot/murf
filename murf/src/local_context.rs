//! The [`local_context`](self) module implements the [`LocalContext`] type.

use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Weak;

use parking_lot::Mutex;

use crate::Expectation;

/// Type that can be used to manage and store call expectations to a associated
/// or static function in a thread local context. If no [`LocalContext`] is
/// provided when the expectation was defined the expectation is stored in the
/// global expectation context.
///
/// Local contexts may be useful if you want to use `murf` in a threaded environment
/// to execute tests in parallel. Using the global context the expectations of
/// multiple tests may be mixed which result in undefined behavior.
#[must_use]
#[derive(Debug)]
pub struct LocalContext {
    _marker: PhantomData<()>,
}

/// Actual state that is stored for the [`LocalContext`].
#[derive(Debug)]
pub struct Inner {
    parent: Option<Box<Inner>>,
    expectations: HashMap<usize, Vec<WeakException>>,
}

type WeakException = Weak<Mutex<Box<dyn Expectation + Send + Sync + 'static>>>;

impl LocalContext {
    /// Create a new [`LocalContext`] instance.
    pub fn new() -> Self {
        CURRENT_CONTEXT.with(|cell| {
            let mut cell = cell.borrow_mut();
            let parent = cell.take().map(Box::new);

            *cell = Some(Inner {
                parent,
                expectations: HashMap::new(),
            });
        });

        Self {
            _marker: PhantomData,
        }
    }

    /// Get the state of the current local context.
    pub fn current() -> Rc<RefCell<Option<Inner>>> {
        CURRENT_CONTEXT.with(Clone::clone)
    }
}

impl Default for LocalContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LocalContext {
    fn drop(&mut self) {
        CURRENT_CONTEXT.with(|cell| {
            let mut cell = cell.borrow_mut();
            *cell = cell.take().unwrap().parent.map(|x| *x);
        });
    }
}

impl Inner {
    /// Return a iterator of expectations that are defined in the current thread local context.
    pub fn expectations(&self, type_id: usize) -> impl Iterator<Item = &'_ WeakException> + '_ {
        let parent: Box<dyn Iterator<Item = &WeakException>> = Box::new(
            self.parent
                .as_ref()
                .into_iter()
                .flat_map(move |p| p.expectations(type_id)),
        );

        self.expectations
            .get(&type_id)
            .into_iter()
            .flat_map(|x| x.iter())
            .chain(parent)
    }

    /// Add a expectation the the current thread local context.
    pub fn push(&mut self, type_id: usize, expectation: WeakException) {
        self.expectations
            .entry(type_id)
            .or_default()
            .push(expectation);
    }
}

thread_local! {
    static CURRENT_CONTEXT: Rc<RefCell<Option<Inner>>> = Rc::new(RefCell::new(None));
}
