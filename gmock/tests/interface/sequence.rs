use gmock::{expect_method_call, matcher::eq, mock, Sequence};

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
    let (handle, mock) = MyStruct::mock_with_handle();

    expect_method_call!(handle as Fuu, fuu(eq(1))).in_sequence(&seq);
    expect_method_call!(handle as Fuu, fuu(eq(2))).in_sequence(&seq);

    mock.fuu(1);
    mock.fuu(2);
}

#[test]
#[should_panic]
fn failure() {
    let seq = Sequence::default();
    let (handle, mock) = MyStruct::mock_with_handle();

    expect_method_call!(handle as Fuu, fuu(eq(1))).in_sequence(&seq);
    expect_method_call!(handle as Fuu, fuu(eq(2))).in_sequence(&seq);

    mock.fuu(2);
    mock.fuu(1);
}

#[test]
fn multi_sequence() {
    let seq0 = Sequence::default();
    let seq1 = Sequence::default();

    let (handle, mock) = MyStruct::mock_with_handle();

    expect_method_call!(handle as Fuu, fuu(eq(1))).add_sequence(&seq0);
    expect_method_call!(handle as Fuu, fuu(eq(2))).add_sequence(&seq1);
    expect_method_call!(handle as Fuu, fuu(eq(3)))
        .add_sequence(&seq0)
        .add_sequence(&seq1);
    mock.fuu(1);
    mock.fuu(2);
    mock.fuu(3);
    handle.checkpoint();

    expect_method_call!(handle as Fuu, fuu(eq(1))).add_sequence(&seq0);
    expect_method_call!(handle as Fuu, fuu(eq(2))).add_sequence(&seq1);
    expect_method_call!(handle as Fuu, fuu(eq(3)))
        .add_sequence(&seq0)
        .add_sequence(&seq1);
    mock.fuu(2);
    mock.fuu(1);
    mock.fuu(3);
    handle.checkpoint();
}
