use murf::{expect_call, matcher::Eq, mock, InSequence};

trait Fuu {
    fn fuu(&self, arg: usize);
}

mock! {
    #[derive(Default, Clone)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(&self, _arg: usize);
    }
}

#[test]
fn success() {
    let (handle, mock1) = MyStruct::mock();

    let _seq = InSequence::default();
    expect_call!(handle as Fuu, fuu(Eq(1))).times(1);
    expect_call!(handle as Fuu, fuu(Eq(2))).times(1);

    let mock2 = mock1.clone();

    mock1.fuu(1);
    mock2.fuu(2);
}
