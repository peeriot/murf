use gmock::{action::Return, expect_method_call, matcher::eq, mock};

trait Fuu {
    fn fuu(&self, x: usize) -> usize;
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(&self, _x: usize) -> usize;
    }
}

struct Service<T: Fuu> {
    fuu: T,
}

impl<T: Fuu> Service<T> {
    fn new(fuu: T) -> Self {
        Self { fuu }
    }

    fn exec(&self) -> usize {
        self.fuu.fuu(4)
    }
}

#[test]
fn success() {
    let (handle, mock) = MyStruct::mock_with_handle();

    let service = Service::new(mock);

    expect_method_call!(handle as Fuu, fuu(eq(4))).will_once(Return(4));

    assert_eq!(4, service.exec());
}

#[test]
#[should_panic]
fn failure() {
    let (handle, mock) = MyStruct::mock_with_handle();

    let service = Service::new(mock);

    expect_method_call!(handle as Fuu, fuu(_)).will_once(Return(4));

    drop(service);
}
