use murf::{expect_call, mock};

#[derive(Default, Clone)]
pub struct MyStruct;

mock! {
    impl Clone for MyStruct {
        fn clone(&self) -> Self;
    }
}

#[test]
fn success() {
    let (handle, mock) = MyStruct::mock();

    expect_call!(handle as Clone, clone());

    let _ = mock.clone();
}
