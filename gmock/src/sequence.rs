use std::cell::UnsafeCell;
use std::sync::Arc;

use parking_lot::Mutex;

/* Sequence */

#[derive(Default, Debug, Clone)]
pub struct Sequence {
    inner: Arc<Mutex<Inner>>,
}

impl Sequence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_handle(&self) -> SequenceHandle {
        Inner::create_handle(self.inner.clone())
    }
}

/* SequenceHandle */

pub struct SequenceHandle {
    id: usize,
    inner: Arc<Mutex<Inner>>,
}

impl SequenceHandle {
    pub fn check(&self) -> bool {
        self.inner.lock().check(self.id)
    }

    pub fn set_ready(&self) {
        self.inner.lock().set_ready(self.id);
    }
}

/* InSequence */

#[derive(Debug)]
pub struct InSequence {
    parent: Option<Arc<Mutex<Inner>>>,
}

impl InSequence {
    pub fn new(sequence: &Sequence) -> Self {
        Self::new_with(sequence.inner.clone())
    }

    pub fn create_handle() -> Option<SequenceHandle> {
        CURRENT_SEQUENCE.with(|cell| {
            unsafe { &*cell.get() }
                .as_ref()
                .map(|inner| Inner::create_handle(inner.clone()))
        })
    }

    fn new_with(inner: Arc<Mutex<Inner>>) -> Self {
        let parent = CURRENT_SEQUENCE.with(|cell| unsafe { &mut *cell.get() }.replace(inner));

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
        CURRENT_SEQUENCE.with(|cell| unsafe { *cell.get() = self.parent.take() });
    }
}

/* Inner */

#[derive(Debug)]
struct Inner {
    next_id: usize,
    current_id: usize,
    current_is_ready: bool,
}

impl Inner {
    fn create_handle(inner: Arc<Mutex<Self>>) -> SequenceHandle {
        let id = inner.lock().next_id();

        SequenceHandle { id, inner }
    }

    fn next_id(&mut self) -> usize {
        let ret = self.next_id;

        self.next_id += 1;

        ret
    }

    fn check(&mut self, id: usize) -> bool {
        if self.current_id == id {
            true
        } else if self.current_id + 1 == id && self.current_is_ready {
            self.current_id += 1;
            self.current_is_ready = false;

            true
        } else {
            false
        }
    }

    fn set_ready(&mut self, id: usize) {
        if self.current_id == id {
            self.current_is_ready = true;
        }
    }
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            next_id: 1,
            current_id: 0,
            current_is_ready: true,
        }
    }
}

thread_local! {
    static CURRENT_SEQUENCE: UnsafeCell<Option<Arc<Mutex<Inner>>>> = UnsafeCell::new(None);
}
