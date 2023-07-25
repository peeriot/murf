use gmock::{expect_call, matcher::eq, mock, InSequence};

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
    let (handle, mock) = MyStruct::mock_with_handle();

    let _seq = InSequence::default();
    expect_call!(handle as Fuu, fuu(eq(1)));
    expect_call!(handle as Fuu, fuu(eq(2)));

    mock.fuu(1);
    mock.fuu(2);
}

#[test]
fn remove_sequences() {
    let (handle, mock) = MyStruct::mock_with_handle();

    let _seq = InSequence::default();
    expect_call!(handle as Fuu, fuu(eq(1)));
    expect_call!(handle as Fuu, fuu(eq(2))).no_sequences();
    expect_call!(handle as Fuu, fuu(eq(3)));

    mock.fuu(1);
    mock.fuu(3);

    mock.fuu(2);
}

#[test]
#[should_panic]
fn failure() {
    let (handle, mock) = MyStruct::mock_with_handle();

    let _seq = InSequence::default();
    expect_call!(handle as Fuu, fuu(eq(1)));
    expect_call!(handle as Fuu, fuu(eq(2)));

    mock.fuu(2);
    mock.fuu(1);
}
