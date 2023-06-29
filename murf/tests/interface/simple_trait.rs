use murf::{action::Return, expect_call, matcher::eq, mock};

trait MyTrait {
    fn exec(&self, x: usize) -> usize;
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl MyTrait for MyStruct {
        fn exec(&self, _x: usize) -> usize;
    }
}

struct Service<T: MyTrait> {
    inner: T,
}

impl<T: MyTrait> Service<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }

    fn exec(&self) -> usize {
        self.inner.exec(4)
    }
}

#[test]
fn success() {
    let (handle, mock) = MyStruct::mock();

    let service = Service::new(mock);

    expect_call!(handle as MyTrait, exec(eq(4))).will_once(Return(4));

    assert_eq!(4, service.exec());
}

#[test]
#[should_panic]
fn failure() {
    let (handle, mock) = MyStruct::mock();

    let service = Service::new(mock);

    expect_call!(handle as MyTrait, exec(_)).will_once(Return(4));

    drop(service);
}
