use gmock::{action::Return, expect_call, matcher::eq, mock, LocalContext};

trait Fuu {
    fn fuu(x: usize) -> usize;
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(_x: usize) -> usize;
    }
}

#[test]
fn success() {
    let local_context = LocalContext::new();

    let (handle, _mock) = MyStruct::mock_with_handle();

    expect_call!(handle as Fuu, fuu(eq(4))).will_once(Return(4));

    let type_id = *mock_impl_my_struct::mock_trait_fuu_method_fuu::TYPE_ID;
    let expectations = LocalContext::current()
        .borrow_mut()
        .as_ref()
        .unwrap()
        .expectations(type_id)
        .count();
    assert_eq!(expectations, 1);

    assert_eq!(4, MyStructMock::fuu(4));

    drop(local_context);
}
