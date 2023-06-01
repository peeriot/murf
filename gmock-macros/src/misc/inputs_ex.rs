use quote::ToTokens;
use syn::{punctuated::Punctuated, FnArg, PatType, Token};

pub trait InputsEx {
    type Iter<'x>: Iterator<Item = &'x PatType> + 'x
    where
        Self: 'x;

    fn without_self_arg(&self) -> Self::Iter<'_>;
}

impl InputsEx for Punctuated<FnArg, Token![,]> {
    type Iter<'x> = Box<dyn Iterator<Item = &'x PatType> + 'x>;

    fn without_self_arg(&self) -> Self::Iter<'_> {
        Box::new(self.iter().filter_map(|i| match i {
            FnArg::Receiver(_) => None,
            FnArg::Typed(t) if t.pat.to_token_stream().to_string() == "self" => None,
            FnArg::Typed(t) => Some(t),
        }))
    }
}
