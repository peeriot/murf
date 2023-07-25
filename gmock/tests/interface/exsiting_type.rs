use std::fmt::Debug;
use std::marker::PhantomData;

use gmock::{action::Return, expect_method_call, mock};

trait Fuu {
    fn fuu(&self) -> usize;
}

#[derive(Default)]
pub struct MyStruct<T: Debug>(PhantomData<T>);

impl<T: Debug> Fuu for MyStruct<T> {
    fn fuu(&self) -> usize {
        6
    }
}

mock! {
    impl<T: Debug> Fuu for MyStruct<T> {
        fn fuu(&self) -> usize;
    }
}

#[test]
fn success() {
    let (handle, mock) = MyStruct::<usize>::mock_with_handle();

    expect_method_call!(handle as Fuu, fuu()).times(1);
    expect_method_call!(handle as Fuu, fuu()).will_once(Return(4));

    assert_eq!(6, mock.fuu());
    assert_eq!(4, mock.fuu());
}
