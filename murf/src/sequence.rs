use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};

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

#[derive(Debug)]
pub struct SequenceHandle {
    id: usize,
    inner: Arc<Mutex<Inner>>,
    sequence_id: usize,
}

impl SequenceHandle {
    pub fn is_active(&self) -> bool {
        self.inner.lock().is_active(self.id)
    }

    pub fn is_done(&self) -> bool {
        self.inner.lock().is_done(self.id)
    }

    pub fn sequence_id(&self) -> usize {
        self.sequence_id
    }

    pub fn set_ready(&self) {
        self.inner.lock().set_ready(self.id);
    }

    pub fn set_done(&self) {
        self.inner.lock().set_done(self.id);
    }

    pub fn set_description(&self, value: String) {
        self.inner.lock().set_description(self.id, value);
    }

    pub fn unsatisfied(&self) -> Unsatisfied<'_> {
        Unsatisfied::new(self)
    }
}

impl Drop for SequenceHandle {
    fn drop(&mut self) {
        self.inner.lock().set_ready(self.id);
    }
}

/* Unsatisfied */

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

/* Inner */

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
