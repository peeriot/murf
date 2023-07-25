use std::marker::PhantomData;

use gmock::{expect_method_call, matcher::Closure, mock};

pub struct Event<'a>(PhantomData<&'a ()>);

trait Fuu<E> {
    fn fuu(&self, x: E);
}

mock! {
    #[derive(Default)]
    pub struct Handler;

    impl<'a> Fuu<Event<'a>> for Handler {
        fn fuu(&self, _x: Event<'a>);
    }
}

#[test]
fn success() {
    let (handle, mock) = Handler::mock_with_handle();

    expect_method_call!(handle as Fuu<Event<'a>>, fuu(Closure(|_: &Event<'_>| true)));

    mock.fuu(Event(PhantomData));
}
