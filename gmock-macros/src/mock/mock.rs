use quote::{quote, ToTokens};
use syn::{Generics, ImplItem, ItemImpl, Path};

use crate::misc::GenericsEx;

use super::context::{Context, ContextData};

pub struct Mock {
    context: Context,
    impls: Vec<Impl>,
}

struct Impl {
    ga: Generics,
    trait_: Option<Path>,
    items: Vec<ImplItem>,
}

impl Mock {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            impls: Vec::new(),
        }
    }

    pub fn add_impl(&mut self, impl_: &ItemImpl, items: Vec<ImplItem>) {
        let ga = impl_.generics.clone().add_lifetime("'mock");
        let trait_ = impl_.trait_.as_ref().map(|(_, p, _)| p).cloned();

        self.impls.push(Impl { ga, trait_, items });
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
            let Impl { ga, trait_, items } = impl_;

            let trait_ = trait_.as_ref().map(|t| quote!( #t for ));

            let (ga_impl, _ga_types, _ga_where) = ga.split_for_impl();

            quote! {
                impl #ga_impl #trait_ Mock #ga_mock_types #ga_mock_where {
                    #( #items )*
                }
            }
        });

        tokens.extend(quote! {
            pub struct Mock #ga_mock_impl #ga_mock_where {
                pub state: #ident_state #ga_state_types,
                pub shared: Arc<Mutex<Shared #ga_mock_types>>,
                pub handle: Option<Handle #ga_handle_types>
            }

            impl #ga_mock_impl Mock #ga_mock_types #ga_mock_where {
                pub fn from_state(state: #ident_state #ga_state_types) -> Self {
                    let shared = Arc::new(Mutex::new(Shared::default()));
                    let handle = Handle {
                        shared: shared.clone(),
                        check_on_drop: true,
                    };
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
