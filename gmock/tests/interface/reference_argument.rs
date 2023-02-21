use gmock::{action::Return, expect_call, matcher::eq, mock};

trait Fuu {
    fn fuu(&self, x: &usize) -> &usize;
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(&self, x: &usize) -> &usize;
    }
}

#[test]
fn success() {
    let (handle, mock) = MyStruct::mock();

    expect_call!(handle as Fuu, fuu(eq(&6))).will_once(Return(&4));

    assert_eq!(&4, mock.fuu(&6));
}
