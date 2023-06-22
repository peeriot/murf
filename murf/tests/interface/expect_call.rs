use murf::{expect_call, mock};

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
    let (handle, _mock) = MyStruct::mock();

    expect_call!(handle as Fuu, fuu()).times(0);

    let tuple = (handle, 0);

    expect_call!(tuple.0 as Fuu, fuu()).times(0);
}
