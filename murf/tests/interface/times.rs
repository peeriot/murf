use murf::{expect_method_call, mock, InSequence};

trait Fuu {
    fn fuu(&self);
}

trait Bar {
    fn bar(&self);
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
    pub struct MockedBar;

    impl Bar for MockedBar {
        fn bar(&self);
    }
}

#[test]
fn success() {
    let (fuu, fuu_mock) = MockedFuu::mock_with_handle();

    expect_method_call!(fuu as Fuu, fuu()).times(1);
    fuu_mock.fuu();
    fuu.checkpoint();

    expect_method_call!(fuu as Fuu, fuu()).times(1..4);
    fuu_mock.fuu();
    fuu_mock.fuu();
    fuu_mock.fuu();
    fuu.checkpoint();

    expect_method_call!(fuu as Fuu, fuu()).times(1..=3);
    fuu_mock.fuu();
    fuu_mock.fuu();
    fuu_mock.fuu();
    fuu.checkpoint();

    expect_method_call!(fuu as Fuu, fuu()).times(2..);
    fuu_mock.fuu();
    fuu_mock.fuu();
    fuu.checkpoint();

    expect_method_call!(fuu as Fuu, fuu()).times(..2);
    fuu_mock.fuu();
    fuu.checkpoint();

    expect_method_call!(fuu as Fuu, fuu()).times(..=2);
    fuu_mock.fuu();
    fuu_mock.fuu();
    fuu.checkpoint();
}

#[test]
fn zero_or_one() {
    let _sequence = InSequence::default();

    {
        let (fuu, fuu_mock) = MockedFuu::mock_with_handle();
        let (bar, bar_mock) = MockedBar::mock_with_handle();

        expect_method_call!(fuu as Fuu, fuu()).times(1);
        expect_method_call!(bar as Bar, bar()).times(0..=1);
        expect_method_call!(fuu as Fuu, fuu()).times(1);

        fuu_mock.fuu();
        bar_mock.bar();
        fuu_mock.fuu();
    }

    {
        let (fuu, fuu_mock) = MockedFuu::mock_with_handle();
        let (bar, _bar_mock) = MockedBar::mock_with_handle();

        expect_method_call!(fuu as Fuu, fuu()).times(1);
        expect_method_call!(bar as Bar, bar()).times(0..=1);
        expect_method_call!(fuu as Fuu, fuu()).times(1);

        fuu_mock.fuu();
        fuu_mock.fuu();
    }
}
