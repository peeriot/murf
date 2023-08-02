use std::marker::PhantomData;

use gmock::{action::Return, mock};
use gmock_macros::expect_method_call;

trait Fuu {
    fn fuu(&self);
}

trait Bar {
    type Fuu: Fuu;

    fn bar(&self) -> &Self::Fuu;
}

mock! {
    #[derive(Default)]
    pub struct MockedFuu;

    impl Fuu for MockedFuu {
        fn fuu(&self);
    }
}

mock! {
    #[derive(Default)]
    pub struct MockedBar<'mock, 'fuu>
    where
        'fuu: 'mock,
    {
        mock_lt: PhantomData<&'mock ()>,
        fuu_lt: PhantomData<&'fuu ()>,
    }

    impl<'mock, 'fuu> Bar for MockedBar<'mock, 'fuu>
    where
        'fuu: 'mock,
    {
        type Fuu = MockedFuuMock<'fuu>;

        fn bar(&self) -> &MockedFuuMock<'fuu>;
    }
}

struct Test<'mock, 'fuu>
where
    'fuu: 'mock,
{
    bar: MockedBarHandle<'mock, 'fuu>,
    bar_mock: MockedBarMock<'mock, 'fuu>,
}

impl<'mock, 'fuu> Test<'mock, 'fuu>
where
    'fuu: 'mock,
{
    fn new() -> Test<'mock, 'fuu> {
        let (bar, bar_mock) = MockedBar::mock_with_handle();

        Test { bar, bar_mock }
    }
}

#[test]
fn test() {
    let (fuu, fuu_mock) = MockedFuu::mock_with_handle();
    let Test { bar, bar_mock } = Test::new();

    expect_method_call!(bar as Bar, bar()).will_once(Return(&fuu_mock));
    expect_method_call!(fuu as Fuu, fuu());

    bar_mock.bar().fuu();
}
