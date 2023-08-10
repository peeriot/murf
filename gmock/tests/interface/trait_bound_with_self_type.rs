use gmock_macros::mock;

pub trait Spawner<T> {
    fn spawn(task: T);
}

pub trait Spawnable<T> {
    fn spawn(self, spawner: T);
}

mock! {
    pub struct MockedTask;

    impl<T> Spawnable<T> for MockedTask
    where
        T: Spawner<Self>
    {
        #[gmock(no_default_impl)]
        fn spawn(self, spawner: T);
    }
}
