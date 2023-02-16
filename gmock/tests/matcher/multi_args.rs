use gmock::{expect_call, matcher::eq, mock};

trait Fuu {
    fn fuu(&self, x: usize, y: usize, z: usize);
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(&self, x: usize, y: usize, z: usize);
    }
}

#[test]
fn success() {
    let (handle, mock) = MyStruct::mock();

    expect_call!(handle as Fuu, fuu(eq(4), eq(4), eq(4)));

    mock.fuu(4, 4, 4);
}
