use super::Action;

pub fn invoke<F>(func: F) -> Invoke<F> {
    Invoke(func)
}

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
