use std::pin::Pin;
use std::task::{Context, Poll};

use futures::task::noop_waker_ref;
use futures::Stream;
use gmock::{action::Return, expect_call, mock};

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Stream for MyStruct {
        type Item = usize;

        fn poll_next(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>
        ) -> Poll<Option<usize>>;
    }
}

#[test]
fn success() {
    let (handle, mut mock) = MyStruct::mock();

    expect_call!(handle as Stream, poll_next(_)).will_once(Return(Poll::Ready(None)));

    let mut cx = Context::from_waker(noop_waker_ref());
    assert_eq!(Poll::Ready(None), Pin::new(&mut mock).poll_next(&mut cx));
}
