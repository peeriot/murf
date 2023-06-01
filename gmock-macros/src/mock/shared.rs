use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::misc::GenericsEx;

use super::context::{Context, ContextData, MethodContext};

pub struct Shared {
    context: Context,
    expectations: Vec<Ident>,
}

impl Shared {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            expectations: Vec::new(),
        }
    }

    pub fn add_expectation(&mut self, context: &MethodContext) {
        self.expectations
            .push(context.ident_expectation_field.clone());
    }
}

impl ToTokens for Shared {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            context,
            expectations,
        } = self;

        let ContextData {
            ident_state,
            ga_mock,
            trait_send,
            trait_sync,
            ..
        } = &**context;

        let ga_mock_phantom = ga_mock.make_phantom_data();
        let (ga_mock_impl, ga_mock_types, ga_mock_where) = ga_mock.split_for_impl();

        let expectation_field_defs = expectations.iter().map(|field| {
            quote! {
                #field: Vec<Box<dyn gmock::Expectation #trait_send #trait_sync + 'mock>>
            }
        });

        let expectation_field_ctor = expectations.iter().map(|field| {
            quote! {
                #field: Vec::new()
            }
        });

        let expectation_err = format!("Mocked object '{ident_state}' has unfulfilled expectations");

        tokens.extend(quote! {
            pub struct Shared #ga_mock_types #ga_mock_where {
                #( #expectation_field_defs, )*
                _marker: #ga_mock_phantom,
            }

            impl #ga_mock_impl Shared #ga_mock_types #ga_mock_where {
                pub(super) fn checkpoint(&self) {
                    let mut raise = false;

                    #(

                        for ex in &self.#expectations {
                            if ex.is_ready() {
                                ex.set_done();
                            } else {
                                if !raise {
                                    println!();
                                    println!(#expectation_err);
                                    raise = true;
                                }

                                println!("- {}", &ex);
                            }
                        }

                    )*

                    if raise {
                        println!();
                        panic!(#expectation_err);
                    }
                }
            }

            impl #ga_mock_impl Default for Shared #ga_mock_types #ga_mock_where {
                fn default() -> Self {
                    Self {
                        #( #expectation_field_ctor, )*
                        _marker: PhantomData,
                    }
                }
            }
        });
    }
}
