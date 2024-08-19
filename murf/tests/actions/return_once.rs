use murf::{action::Return, expect_method_call, mock};

#[derive(Debug, Eq, PartialEq)]
pub struct Data(usize);

trait Fuu {
    fn fuu(&self) -> Data;
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(&self) -> Data;
    }
}

#[test]
fn success() {
    let (handle, mock) = MyStruct::mock_with_handle();

    expect_method_call!(handle as Fuu, fuu()).will_once(Return(Data(1)));
    expect_method_call!(handle as Fuu, fuu()).will_once(Return(Data(2)));

    assert_eq!(Data(1), mock.fuu());
    assert_eq!(Data(2), mock.fuu());
}
