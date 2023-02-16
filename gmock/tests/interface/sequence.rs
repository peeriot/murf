use gmock::{expect_call, matcher::eq, mock, Sequence};

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
    let seq = Sequence::default();
    let (handle, mock) = MyStruct::mock();

    expect_call!(handle as Fuu, fuu(eq(1))).in_sequence(&seq);
    expect_call!(handle as Fuu, fuu(eq(2))).in_sequence(&seq);

    mock.fuu(1);
    mock.fuu(2);
}

#[test]
#[should_panic]
fn failure() {
    let seq = Sequence::default();
    let (handle, mock) = MyStruct::mock();

    expect_call!(handle as Fuu, fuu(eq(1))).in_sequence(&seq);
    expect_call!(handle as Fuu, fuu(eq(2))).in_sequence(&seq);

    mock.fuu(2);
    mock.fuu(1);
}
