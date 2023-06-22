use murf::{expect_call, matcher::range, mock};

trait Fuu {
    fn fuu(&self, x: usize);
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(&self, x: usize);
    }
}

#[test]
fn success() {
    let (handle, mock) = MyStruct::mock();

    expect_call!(handle as Fuu, fuu(range(4..=6)));

    mock.fuu(5);
}

#[test]
#[should_panic]
fn failure() {
    let (handle, mock) = MyStruct::mock();

    expect_call!(handle as Fuu, fuu(range(4..=6)));

    mock.fuu(7);
}
