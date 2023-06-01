use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::context::{Context, ContextData};

pub struct Mockable {
    context: Context,
}

impl Mockable {
    pub fn new(context: Context) -> Self {
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
            ..
        } = &**context;

        let (_ga_mock_impl, ga_mock_types, _ga_mock_where) = ga_mock.split_for_impl();
        let (ga_state_impl, ga_state_types, ga_state_where) = ga_state.split_for_impl();
        let (_ga_handle_impl, ga_handle_types, _ga_handle_where) = ga_handle.split_for_impl();

        tokens.extend(quote! {
            pub trait Mockable {
                type Handle<'mock>;
                type Mock<'mock>;

                fn into_mock<'mock>(self) -> (Self::Handle<'mock>, Self::Mock<'mock>);
            }

            impl #ga_state_impl Mockable for #ident_state #ga_state_types #ga_state_where {
                type Handle<'mock> = #ident_module::Handle #ga_handle_types;
                type Mock<'mock> = #ident_module::Mock #ga_mock_types;

                fn into_mock<'mock>(self) -> (Self::Handle<'mock>, Self::Mock<'mock>) {
                    let shared = Arc::new(Mutex::new(#ident_module::Shared::default()));
                    let handle = #ident_module::Handle {
                        shared: shared.clone()
                    };
                    let mock = #ident_module::Mock {
                        state: self,
                        shared,
                    };

                    (handle, mock)
                }
            }
        })
    }
}
