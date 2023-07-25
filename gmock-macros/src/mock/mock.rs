use quote::{quote, ToTokens};
use syn::ImplItem;

use crate::mock::context::ImplContextData;

use super::context::{Context, ContextData, ImplContext};

pub struct Mock {
    context: Context,
    impls: Vec<Impl>,
}

struct Impl {
    context: ImplContext,
    items: Vec<ImplItem>,
}

impl Mock {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            impls: Vec::new(),
        }
    }

    pub fn add_impl(&mut self, context: ImplContext, items: Vec<ImplItem>) {
        self.impls.push(Impl { context, items });
    }
}

impl ToTokens for Mock {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { context, impls } = self;

        let ContextData {
            ident_state,
            ga_state,
            ga_mock,
            ga_handle,
            derive_clone,
            derive_default,
            ..
        } = &**context;

        let (ga_mock_impl, ga_mock_types, ga_mock_where) = ga_mock.split_for_impl();
        let (_ga_state_impl, ga_state_types, _ga_state_where) = ga_state.split_for_impl();
        let (_ga_handle_impl, ga_handle_types, _ga_handle_where) = ga_handle.split_for_impl();

        let mock_clone_impl = derive_clone.then(|| {
            quote! {
                impl #ga_mock_impl Clone for Mock #ga_mock_types #ga_mock_where {
                    fn clone(&self) -> Self {
                        Self {
                            state: self.state.clone(),
                            shared: self.shared.clone(),
                            handle: self.handle.clone(),
                        }
                    }
                }
            }
        });

        let mock_default_impl = derive_default.then(|| {
            quote! {
                impl #ga_mock_impl Mock #ga_mock_types #ga_mock_where {
                    pub fn new() -> Self {
                        Self::from_state(Default::default())
                    }
                }

                impl #ga_mock_impl Default for Mock #ga_mock_types #ga_mock_where {
                    fn default() -> Self {
                        Self::new()
                    }
                }
            }
        });

        let impls = impls.iter().map(|impl_| {
            let Impl { context, items } = impl_;
            let ImplContextData {
                trait_,
                ga_impl_mock,
                ..
            } = &**context;

            let trait_ = trait_.as_ref().map(|t| quote!( #t for ));

            let (ga_impl, _ga_types, _ga_where) = ga_impl_mock.split_for_impl();

            quote! {
                impl #ga_impl #trait_ Mock #ga_mock_types #ga_mock_where {
                    #( #items )*
                }
            }
        });

        tokens.extend(quote! {
            pub struct Mock #ga_mock_impl #ga_mock_where {
                pub state: #ident_state #ga_state_types,
                pub (super) shared: Arc<Mutex<Shared #ga_mock_types>>,
                pub (super) handle: Option<Handle #ga_handle_types>
            }

            impl #ga_mock_impl Mock #ga_mock_types #ga_mock_where {
                pub fn from_state(state: #ident_state #ga_state_types) -> Self {
                    let handle = Handle::new();
                    let shared = handle.shared.clone();
                    let mock = Self {
                        state,
                        shared,
                        handle: Some(handle),
                    };

                    mock
                }

                pub fn mock_handle(&self) -> &Handle #ga_handle_types {
                    if let Some(handle) = &self.handle {
                        handle
                    } else {
                        panic!("The handle of this mock object was already taken!");
                    }
                }

                pub fn mock_split(mut self) -> (Handle #ga_handle_types, Self) {
                    let handle = self.mock_take_handle();

                    (handle, self)
                }

                pub fn mock_take_handle(&mut self) -> Handle #ga_handle_types {
                    if let Some(handle) = self.handle.take() {
                        handle
                    } else {
                        panic!("The handle of this mock object was already taken!");
                    }
                }

                pub fn mock_release_handle(&mut self) {
                    self.mock_take_handle().release();
                }
            }

            #mock_clone_impl
            #mock_default_impl

            #( #impls )*
        })
    }
}
