use std::fmt::{Display, Formatter, Result as FmtResult};

use gmock::{expect_method_call, matcher::eq, mock};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Wrapper<'a, T>(&'a T);

impl<'a, T: Display> Display for Wrapper<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Wrapper({})", &self.0)
    }
}

trait Fuu<T> {
    fn fuu(&self, arg: T);
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl<T> Fuu<T> for MyStruct {
        fn fuu(&self, _arg: T);
    }
}

#[test]
fn success() {
    let fuu = 123usize;
    let fuu = Wrapper(&fuu);
    let (handle, mock) = MyStruct::mock_with_handle();

    expect_method_call!(handle as Fuu, fuu(eq(fuu)));

    mock.fuu(Wrapper(&123usize));
}
