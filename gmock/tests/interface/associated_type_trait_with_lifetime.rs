use std::slice::Iter;

use gmock::{action::Return, expect_method_call, mock};

trait Values<T> {
    type Iter<'x>: Iterator<Item = &'x T>
    where
        T: 'x,
        Self: 'x;

    fn values(&self) -> Self::Iter<'_>;
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Values<usize> for MyStruct {
        type Iter<'x> = Iter<'x, usize>
        where
            Self: 'x;

        fn values(&self) -> Iter<'_, usize>;
    }
}

#[test]
fn success() {
    let values: Vec<usize> = vec![1, 2, 3, 4];

    let (handle, mock) = MyStruct::mock_with_handle::<'_>();

    expect_method_call!(handle as Values<usize>, values()).will_once(Return(values.iter()));

    let values = mock.values().cloned().collect::<Vec<_>>();

    assert_eq!(values, [1, 2, 3, 4]);
}
