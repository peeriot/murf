use gmock::{action::Return, expect_method_call, matcher::eq, mock};

pub trait Fuu<T> {
    fn fuu<X>(&self, value: X) -> T;
}

mock! {
    #[derive(Default, Debug)]
    pub struct MockedFuu;

    impl<T> Fuu<T> for MockedFuu {
        fn fuu<X>(&self, value: X) -> T;
    }
}

#[test]
fn test() {
    let mock = MockedFuu::mock();

    expect_method_call!(mock as Fuu, fuu(eq(123u8))).will_once(Return(312usize));
    expect_method_call!(mock as Fuu<usize>, fuu::<_>(eq(123u8))).will_once(Return(312usize));

    assert_eq!(312usize, mock.fuu(123u8));
    assert_eq!(312usize, mock.fuu(123u8));
}
