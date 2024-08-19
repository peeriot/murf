use std::{cell::RefCell, rc::Rc};

use murf::{action::ReturnPointee, expect_method_call, mock};

trait Fuu {
    fn fuu(&self) -> usize;
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl Fuu for MyStruct {
        fn fuu(&self) -> usize;
    }
}

#[test]
fn success() {
    let val = Rc::new(RefCell::new(5usize));

    let (handle, mock) = MyStruct::mock_with_handle();

    expect_method_call!(handle as Fuu, fuu()).will_once(ReturnPointee(val.clone()));
    expect_method_call!(handle as Fuu, fuu()).will_once(ReturnPointee(val.clone()));

    assert_eq!(5, mock.fuu());

    *val.borrow_mut() = 10;

    assert_eq!(10, mock.fuu());

    drop(handle);
    drop(mock);
}
