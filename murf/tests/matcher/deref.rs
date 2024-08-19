use murf::{
    expect_method_call,
    matcher::{deref, eq},
    mock,
};

trait Fuu {
    fn fuu(&self, x: &usize);
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(&self, _x: &usize);
    }
}

#[test]
fn success() {
    let (handle, mock) = MyStruct::mock_with_handle();

    expect_method_call!(handle as Fuu, fuu(deref(eq(4))));

    mock.fuu(&4);
}
