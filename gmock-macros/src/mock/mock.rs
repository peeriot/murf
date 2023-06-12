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
            derive_clone,
            ..
        } = &**context;

        let (ga_mock_impl, ga_mock_types, ga_mock_where) = ga_mock.split_for_impl();
        let (_ga_state_impl, ga_state_types, _ga_state_where) = ga_state.split_for_impl();

        let mock_default_clone_impl = derive_clone.then(|| {
            quote! {
                impl #ga_mock_impl Clone for Mock #ga_mock_types #ga_mock_where {
                    fn clone(&self) -> Self {
                        Self {
                            state: self.state.clone(),
                            shared: self.shared.clone(),
                        }
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
            }

            #mock_default_clone_impl

            #( #impls )*
        })
    }
}
