use gmock::{expect_call, mock};

trait Fuu {
    fn fuu(&self, x: usize) -> &mut usize;
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(&self, _x: usize) -> &mut usize;
    }
}

#[test]
fn success() {
    let mut i = 5;
    let i_ref = &mut i;

    let (handle, mock) = MyStruct::mock();

    expect_call!(handle as Fuu, fuu(_)).will_once(move |x| {
        assert_eq!(x, 5);

        *i_ref = x;

        i_ref
    });

    mock.fuu(5);

    drop(handle);
    drop(mock);

    assert_eq!(5, i);
}
