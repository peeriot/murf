use super::Action;

/* Return */

pub fn return_<T: Clone>(value: T) -> Return<T> {
    Return(value)
}

pub struct Return<T>(pub T);

impl<T, X> Action<X, T> for Return<T>
where
    T: Clone,
{
    fn exec(self, _args: X) -> T {
        self.0
    }
}

/* ReturnRef */

pub fn return_ref<T>(value: &T) -> ReturnRef<'_, T> {
    ReturnRef(value)
}

pub struct ReturnRef<'a, T>(pub &'a T);

impl<'a, T, X> Action<X, &'a T> for ReturnRef<'a, T> {
    fn exec(self, _args: X) -> &'a T {
        self.0
    }
}
