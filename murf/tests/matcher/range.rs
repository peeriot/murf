use murf::{expect_method_call, matcher::range, mock};

trait Fuu {
    fn fuu(&self, x: usize);
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(&self, _x: usize);
    }
}

#[test]
fn success() {
    let mock = MyStructMock::default();

    expect_method_call!(mock as Fuu, fuu(range(4..=6)));

    mock.fuu(5);
}

#[test]
#[should_panic]
fn failure() {
    let mock = MyStruct::mock();

    expect_method_call!(mock as Fuu, fuu(range(4..=6)));

    mock.fuu(7);
}
