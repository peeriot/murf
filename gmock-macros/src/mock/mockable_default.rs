use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct MockableDefault;

impl ToTokens for MockableDefault {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            pub trait MockableDefault: Mockable {
                fn mock<'mock>() -> (Self::Handle<'mock>, Self::Mock<'mock>);
            }

            impl<X> MockableDefault for X
            where
                X: Mockable + Default,
            {
                fn mock<'mock>() -> (Self::Handle<'mock>, Self::Mock<'mock>) {
                    Self::default().into_mock()
                }
            }
        })
    }
}
