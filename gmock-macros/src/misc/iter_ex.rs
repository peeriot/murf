use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub trait IterEx {
    fn parenthesis(self) -> TokenStream;
}

impl<X> IterEx for X
where
    X: IntoIterator,
    X::Item: ToTokens,
{
    fn parenthesis(self) -> TokenStream {
        let mut count = 0;
        let iter = self.into_iter().inspect(|_| count += 1);

        let ret = quote!(#( #iter ),*);

        match count {
            0 => quote!(()),
            1 => ret,
            _ => quote!((#ret)),
        }
    }
}
