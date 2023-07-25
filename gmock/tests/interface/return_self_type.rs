use gmock::{expect_call, mock};

#[derive(Default, Clone)]
pub struct MyStruct;

mock! {
    impl Clone for MyStruct {
        fn clone(&self) -> Self;
    }
}

#[test]
fn success() {
    let (handle, mock) = MyStruct::mock_with_handle();

    expect_call!(handle as Clone, clone());

    let _ = mock.clone();
}
