use murf::{action::Return, expect_call, expect_method_call, matcher::eq, mock};

trait Fuu: Sized {
    fn new(x: usize) -> Result<Self, ()>;

    fn fuu(&self) -> usize;
}

mock! {
    #[derive(Default, Send, Sync)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn new(x: usize) -> Result<Self, ()>;

        fn fuu(&self) -> usize;
    }
}

#[test]
fn success() {
    let static_handle = MyStructHandle::new();

    let (mock_handle, mock) = MyStruct::mock_with_handle();

    expect_call!(static_handle as Fuu, new(eq(4))).will_once(Return(Ok(mock)));
    expect_method_call!(mock_handle as Fuu, fuu()).will_once(Return(5));

    let my_struct = <MyStructMock as Fuu>::new(4).unwrap();
    assert_eq!(5, my_struct.fuu());
}
