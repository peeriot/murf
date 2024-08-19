use murf::{expect_method_call, mock};

pub trait Fuu {
    fn fuu<const X: usize>(&self);
}

mock! {
    #[derive(Default, Debug)]
    pub struct MockedFuu;

    impl Fuu for MockedFuu {
        fn fuu<const X: usize>(&self);
    }
}

#[test]
fn test() {
    let mock = MockedFuu::mock();

    expect_method_call!(mock as Fuu, fuu::<1024>());

    mock.fuu::<1024>();
}
