use murf::{action::Return, expect_method_call, mock};

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Iterator for MyStruct {
        type Item = usize;

        fn next(&mut self) -> Option<usize>;
    }
}

struct NewIterator<T: Iterator> {
    inner: T,
}

impl<T: Iterator> NewIterator<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }

    fn next(&mut self) -> Option<T::Item> {
        self.inner.next()
    }
}

#[test]
fn success() {
    let (handle, mock) = MyStruct::mock_with_handle();

    let mut iter = NewIterator::new(mock);

    expect_method_call!(handle as Iterator, next()).will_once(Return(Some(2)));

    assert_eq!(Some(2), iter.next());
}
