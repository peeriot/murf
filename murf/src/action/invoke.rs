use super::Action;

/// Creates a [`Invoke`] action that forwards the call to the passed function `f`.
pub fn invoke<F>(func: F) -> Invoke<F> {
    Invoke(func)
}

/// Action that forwards the call to the passed function `F`.
#[derive(Debug)]
pub struct Invoke<F>(pub F);

impl<F, X, T> Action<X, T> for Invoke<F>
where
    F: FnOnce(X) -> T,
{
    fn exec(self, args: X) -> T {
        (self.0)(args)
    }
}
