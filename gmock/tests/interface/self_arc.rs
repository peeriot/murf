use std::sync::Arc;
use std::task::{Wake, Waker};

use gmock::{expect_call, mock};

mock! {
    #[derive(Default, Clone, Send, Sync)]
    pub struct Wakeable;

    impl Wake for Wakeable {
        fn wake(self: Arc<Self>);
        fn wake_by_ref(self: &Arc<Self>);
    }
}

impl WakeableMock<'static> {
    pub fn into_waker(self) -> Waker {
        Waker::from(Arc::new(self))
    }
}

#[test]
fn success() {
    let (wake_handle, wake_mock) = Wakeable::mock();
    let waker = wake_mock.into_waker();

    expect_call!(wake_handle as Wake, wake_by_ref());
    waker.wake_by_ref();

    expect_call!(wake_handle as Wake, wake());
    waker.wake();
}
