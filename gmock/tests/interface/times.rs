use gmock::{expect_method_call, mock};

trait Fuu {
    fn fuu(&self);
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(&self);
    }
}

#[test]
fn success() {
    let (handle, mock) = MyStruct::mock_with_handle();

    expect_method_call!(handle as Fuu, fuu()).times(1);
    mock.fuu();
    handle.checkpoint();

    expect_method_call!(handle as Fuu, fuu()).times(1..4);
    mock.fuu();
    mock.fuu();
    mock.fuu();
    handle.checkpoint();

    expect_method_call!(handle as Fuu, fuu()).times(1..=3);
    mock.fuu();
    mock.fuu();
    mock.fuu();
    handle.checkpoint();

    expect_method_call!(handle as Fuu, fuu()).times(2..);
    mock.fuu();
    mock.fuu();
    handle.checkpoint();

    expect_method_call!(handle as Fuu, fuu()).times(..2);
    mock.fuu();
    handle.checkpoint();

    expect_method_call!(handle as Fuu, fuu()).times(..=2);
    mock.fuu();
    mock.fuu();
    handle.checkpoint();
}
