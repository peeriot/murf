use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{task::noop_waker_ref, Stream};
use murf::{action::Return, expect_method_call, mock};

mock! {
    pub struct MyStruct<T>(PhantomData<T>);

    impl<T> Stream for MyStruct<T> {
        type Item = T;

        fn poll_next(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>
        ) -> Poll<Option<T>>;
    }
}

#[test]
fn success() {
    let (handle, mut mock) = MyStruct(PhantomData::<usize>).into_mock().mock_split();

    expect_method_call!(handle as Stream, poll_next(_)).will_once(Return(Poll::Ready(None)));

    let mut cx = Context::from_waker(noop_waker_ref());
    assert_eq!(Poll::Ready(None), Pin::new(&mut mock).poll_next(&mut cx));
}
