//! The [`sequence`](self) module contains different types and helpers to manage
//! the order of different call expectations.

use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};

/// A sequence is used to manage the order of call expectations.
///
/// The sequence uses a so called [`SequenceHandle`] to keep track of registered
/// elements in the sequence. The handle can be used to check the status of a
/// particular element in the sequence.
///
/// A sequence must explicitly be added to a expectation using the `in_sequence`
/// or `add_sequence` method of the expectation object.
#[must_use]
#[derive(Default, Debug, Clone)]
pub struct Sequence {
    inner: Arc<Mutex<Inner>>,
}

impl Sequence {
    /// Create a new empty sequence.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new handle that is added to the end of the sequence.
    #[must_use]
    pub fn create_handle(&self) -> SequenceHandle {
        Inner::create_handle(self.inner.clone())
    }
}

/// Represents a reserved element in a [`Sequence`].
///
/// The sequence handle can be used to check the status of a particular element
/// in the sequence.
///
/// A handle has different states:
///   - Inactive:   Other handles before the current one are not fulfilled yet.
///   - Active:     The current handle is the one that needs to be processed next.
///   - Ready:      The current handle has been called the expected amount of times,
///                 but may be called more times before marked as done.
///   - Done:       The handle is done and is not expected to be called again in
///                 the future.
#[derive(Debug)]
pub struct SequenceHandle {
    id: usize,
    inner: Arc<Mutex<Inner>>,
    sequence_id: usize,
}

impl SequenceHandle {
    /// Returns `true` if the current handle is active, `false` otherwise.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.inner.lock().is_active(self.id)
    }

    /// Returns `true` if the current handle is done, `false` otherwise.
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.inner.lock().is_done(self.id)
    }

    /// Get the id of the sequence this handle belongs to.
    #[must_use]
    pub fn sequence_id(&self) -> usize {
        self.sequence_id
    }

    /// Mark the current handle as ready.
    pub fn set_ready(&self) {
        self.inner.lock().set_ready(self.id);
    }

    /// Mark the current handle as done.
    pub fn set_done(&self) {
        self.inner.lock().set_done(self.id);
    }

    /// Set the description of the handle.
    pub fn set_description(&self, value: String) {
        self.inner.lock().set_description(self.id, value);
    }

    /// Get an iterator of unsatisfied (not done yet) handles.
    #[must_use]
    pub fn unsatisfied(&self) -> Unsatisfied<'_> {
        Unsatisfied::new(self)
    }
}

impl Drop for SequenceHandle {
    fn drop(&mut self) {
        self.inner.lock().set_ready(self.id);
    }
}

/// [`Iterator`] over unsatisfied (not done yet) handles in a [`Sequence`].
#[derive(Debug)]
pub struct Unsatisfied<'a> {
    guard: MutexGuard<'a, Inner>,
    id_end: usize,
    id_current: usize,
}

impl<'a> Unsatisfied<'a> {
    fn new(seq_handle: &'a SequenceHandle) -> Self {
        let guard = seq_handle.inner.lock();
        let id_end = seq_handle.id;
        let id_current = guard.current_id;

        Self {
            guard,
            id_end,
            id_current,
        }
    }
}

impl<'a> Iterator for Unsatisfied<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.id_current == self.id_end {
                return None;
            }

            let meta = self.guard.items.get(self.id_current)?;
            self.id_current += 1;

            if !meta.is_ready {
                return Some(meta.description.clone());
            }
        }
    }
}

/// Same like [`Sequence`] with the different that all expectations that are added
/// after the [`InSequence`] has been defined are added to this sequence automatically.
///
/// [`InSequence`] is thread local.
#[derive(Debug)]
pub struct InSequence {
    parent: Option<Arc<Mutex<Inner>>>,
}

impl InSequence {
    /// Create a new [`InSequence`] instance that automatically adds expectations
    /// to the passed `sequence`.
    #[must_use]
    pub fn new(sequence: &Sequence) -> Self {
        Self::new_with(sequence.inner.clone())
    }

    /// Returns a new `Some(SequenceHandle)` for a sequence that was defined by [`InSequence`]
    /// before. `None` is returned if no [`InSequence`] instance is known in the current
    /// context.
    #[must_use]
    pub fn create_handle() -> Option<SequenceHandle> {
        CURRENT_SEQUENCE.with(|cell| {
            cell.borrow()
                .as_ref()
                .map(|inner| Inner::create_handle(inner.clone()))
        })
    }

    fn new_with(inner: Arc<Mutex<Inner>>) -> Self {
        let parent = CURRENT_SEQUENCE.with(|cell| cell.borrow_mut().replace(inner));

        Self { parent }
    }
}

impl Default for InSequence {
    fn default() -> Self {
        Self::new_with(Arc::new(Mutex::new(Inner::default())))
    }
}

impl Drop for InSequence {
    fn drop(&mut self) {
        CURRENT_SEQUENCE.with(|cell| *cell.borrow_mut() = self.parent.take());
    }
}

/// Inner state of a sequence.
#[derive(Debug)]
struct Inner {
    items: Vec<Meta>,
    current_id: usize,
    sequence_id: usize,
}

#[derive(Default, Debug)]
struct Meta {
    is_ready: bool,
    description: String,
}

impl Inner {
    fn create_handle(inner: Arc<Mutex<Self>>) -> SequenceHandle {
        let (id, sequence_id) = {
            let mut inner = inner.lock();
            let id = inner.items.len();
            inner.items.push(Meta::default());

            (id, inner.sequence_id)
        };

        SequenceHandle {
            id,
            inner,
            sequence_id,
        }
    }

    fn is_active(&mut self, id: usize) -> bool {
        loop {
            if self.current_id == id {
                return true;
            } else if self.current_id < id && self.item(self.current_id).is_ready {
                self.current_id += 1;
            } else {
                return false;
            }
        }
    }

    fn is_done(&mut self, id: usize) -> bool {
        self.current_id > id
    }

    fn set_ready(&mut self, id: usize) {
        self.item_mut(id).is_ready = true;
    }

    fn set_done(&mut self, id: usize) {
        if self.current_id == id {
            self.current_id += 1;
        }
    }

    fn set_description(&mut self, id: usize, value: String) {
        self.item_mut(id).description = value;
    }

    fn item(&self, id: usize) -> &Meta {
        self.items.get(id).expect("Invalid sequence handle")
    }

    fn item_mut(&mut self, id: usize) -> &mut Meta {
        self.items.get_mut(id).expect("Invalid sequence handle")
    }
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            sequence_id: SEQUENCE_ID.fetch_add(1, Ordering::Relaxed),
            items: vec![Meta {
                is_ready: true,
                description: "root".into(),
            }],
            current_id: 0,
        }
    }
}

static SEQUENCE_ID: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static CURRENT_SEQUENCE: RefCell<Option<Arc<Mutex<Inner>>>> = const { RefCell::new(None) };
}
