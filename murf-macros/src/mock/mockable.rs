use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::context::{Context, ContextData};

pub(crate) struct Mockable {
    context: Context,
}

impl Mockable {
    pub(crate) fn new(context: Context) -> Self {
        Self { context }
    }
}

impl ToTokens for Mockable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { context } = self;

        let ContextData {
            ident_state,
            ident_module,
            ga_mock,
            ga_state,
            ga_handle,
            extern_mock_lifetime,
            ..
        } = &**context;

        let (_ga_mock_impl, ga_mock_types, _ga_mock_where) = ga_mock.split_for_impl();
        let (ga_state_impl, ga_state_types, ga_state_where) = ga_state.split_for_impl();
        let (_ga_handle_impl, ga_handle_types, _ga_handle_where) = ga_handle.split_for_impl();

        if *extern_mock_lifetime {
            tokens.extend(quote! {
                /// Helper trait that is used to convert a object into it's mocked version.
                pub trait Mockable<'mock> {
                    /// Mocked version of the object.
                    type Mock;

                    /// Handle to control the mock object.
                    type Handle;

                    /// Returns a mocked version of the object.
                    fn into_mock(self) -> Self::Mock;

                    /// Returns a handle and a mocked version of the object.
                    fn into_mock_with_handle(self) -> (Self::Handle, Self::Mock);
                }

                impl #ga_state_impl Mockable<'mock> for #ident_state #ga_state_types #ga_state_where {
                    type Mock = #ident_module::Mock #ga_mock_types;
                    type Handle = #ident_module::Handle #ga_handle_types;

                    fn into_mock(self) -> Self::Mock {
                        Self::Mock::from_state(self)
                    }

                    fn into_mock_with_handle(self) -> (Self::Handle, Self::Mock) {
                        self.into_mock().mock_split()
                    }
                }
            });
        } else {
            tokens.extend(quote! {
                /// Helper trait that is used to convert a object into it's mocked version.
                pub trait Mockable {
                    /// Mocked version of the object.
                    type Mock<'mock>;

                    /// Handle to control the mock object.
                    type Handle<'mock>;

                    /// Returns a mocked version of the object.
                    fn into_mock<'mock>(self) -> Self::Mock<'mock>;

                    /// Returns a handle and a mocked version of the object.
                    fn into_mock_with_handle<'mock>(self) -> (Self::Handle<'mock>, Self::Mock<'mock>);
                }

                impl #ga_state_impl Mockable for #ident_state #ga_state_types #ga_state_where {
                    type Mock<'mock> = #ident_module::Mock #ga_mock_types;
                    type Handle<'mock> = #ident_module::Handle #ga_handle_types;

                    fn into_mock<'mock>(self) -> Self::Mock<'mock> {
                        Self::Mock::from_state(self)
                    }

                    fn into_mock_with_handle<'mock>(self) -> (Self::Handle<'mock>, Self::Mock<'mock>) {
                        self.into_mock().mock_split()
                    }
                }
            });
        }
    }
}
