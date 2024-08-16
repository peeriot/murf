use murf::{action::Return, expect_call, matcher::eq, mock};

trait Fuu {
    fn fuu(x: usize) -> usize;
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(_x: usize) -> usize;
    }
}

#[test]
fn success() {
    let (handle, _mock) = MyStruct::mock_with_handle();

    expect_call!(handle as Fuu, fuu(eq(4))).will_once(Return(4));

    assert_eq!(4, MyStructMock::fuu(4));
}
