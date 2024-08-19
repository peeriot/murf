use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::context::{Context, ContextData};

pub(crate) struct MockableDefault {
    pub context: Context,
}

impl MockableDefault {
    pub(crate) fn new(context: Context) -> Self {
        Self { context }
    }
}

impl ToTokens for MockableDefault {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { context } = self;

        let ContextData {
            extern_mock_lifetime,
            ..
        } = &**context;

        if *extern_mock_lifetime {
            tokens.extend(quote! {
                /// Helper trait that is used to generate a default mock object
                /// from it's mocked type.
                pub trait MockableDefault<'mock>: Mockable<'mock> {
                    /// Returns a new empty mock object.
                    fn mock() -> Self::Mock;

                    /// Returns a new empty handle and a mocked object.
                    fn mock_with_handle() -> (Self::Handle, Self::Mock);
                }

                impl<'mock, X> MockableDefault<'mock> for X
                where
                    X: Mockable<'mock> + Default,
                {
                    fn mock() -> Self::Mock {
                        Self::default().into_mock()
                    }

                    fn mock_with_handle() -> (Self::Handle, Self::Mock) {
                        Self::default().into_mock_with_handle()
                    }
                }
            });
        } else {
            tokens.extend(quote! {
                /// Helper trait that is used to generate a default mock object
                /// from it's mocked type.
                pub trait MockableDefault: Mockable {
                    /// Returns a new empty mock object.
                    fn mock<'mock>() -> Self::Mock<'mock>;

                    /// Returns a new empty handle and a mocked object.
                    fn mock_with_handle<'mock>() -> (Self::Handle<'mock>, Self::Mock<'mock>);
                }

                impl<X> MockableDefault for X
                where
                    X: Mockable + Default,
                {
                    fn mock<'mock>() -> Self::Mock<'mock> {
                        Self::default().into_mock()
                    }

                    fn mock_with_handle<'mock>() -> (Self::Handle<'mock>, Self::Mock<'mock>) {
                        Self::default().into_mock_with_handle()
                    }
                }
            });
        }
    }
}
