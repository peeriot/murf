use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{misc::GenericsEx, mock::context::MethodContextData};

use super::context::{Context, ContextData, MethodContext};

pub struct Shared {
    context: Context,
    expectations: Vec<MethodContext>,
}

impl Shared {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            expectations: Vec::new(),
        }
    }

    pub fn add_expectation(&mut self, context: MethodContext) {
        self.expectations.push(context);
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

        let expectation_field_defs = expectations.iter().map(|cx| {
            let field = &cx.ident_expectation_field;

            if cx.is_associated {
                quote! {
                    #field: Vec<Arc<Mutex<Box<dyn gmock::Expectation + Send + Sync + 'static>>>>
                }
            } else {
                quote! {
                    #field: Vec<Box<dyn gmock::Expectation #trait_send #trait_sync + 'mock>>
                }
            }
        });

        let expectation_field_ctor = expectations.iter().map(|cx| {
            let field = &cx.ident_expectation_field;

            quote! {
                #field: Vec::new()
            }
        });

        let expectation_err = format!("Mocked object '{ident_state}' has unfulfilled expectations");

        let expectations = expectations.iter().map(|cx| {
            let MethodContextData {
                is_associated,
                ident_expectation_field,
                ident_expectation_module,
                ..
            } = &**cx;

            let expectation_unwrap = is_associated.then(|| {
                quote! {
                    let ex = ex.lock();
                }
            });

            let expectation_cleanup = is_associated.then(|| {
                quote! {
                    #ident_expectation_module::cleanup_associated_expectations();
                }
            });

            quote! {
                for ex in &self.#ident_expectation_field {
                    #expectation_unwrap
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

                self.#ident_expectation_field.clear();

                #expectation_cleanup
            }
        });

        tokens.extend(quote! {
            pub struct Shared #ga_mock_types #ga_mock_where {
                #( #expectation_field_defs, )*
                _marker: #ga_mock_phantom,
            }

            impl #ga_mock_impl Shared #ga_mock_types #ga_mock_where {
                pub(super) fn checkpoint(&mut self) {
                    let mut raise = false;

                    #( #expectations )*

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
