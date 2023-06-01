use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Generics;

use crate::misc::GenericsEx;

use super::context::{Context, ContextData, MethodContext, MethodContextData};

pub struct Handle {
    context: Context,
    methods: Vec<MethodContext>,
    ga_handle_extra: Generics,
}

impl Handle {
    pub fn new(context: Context) -> Self {
        let ga_handle_extra = context.ga_handle.clone().add_lifetime_clauses("'mock");

        Self {
            context,
            methods: Vec::new(),
            ga_handle_extra,
        }
    }

    pub fn add_method(&mut self, context: MethodContext) {
        self.methods.push(context);
    }

    fn render_method(&self, context: &MethodContextData) -> TokenStream {
        let MethodContextData {
            ident_expect_method,
            ident_expectation_field,
            ident_expectation_module,
            ga_method,
            ga_expectation,
            ..
        } = context;

        let ga_builder = ga_expectation.clone().add_lifetime("'_");

        let (ga_method_impl, _ga_method_types, ga_method_where) = ga_method.split_for_impl();
        let (_ga_builder_impl, ga_builder_types, _ga_builder_where) = ga_builder.split_for_impl();
        let (_ga_expectation_impl, ga_expectation_types, _ga_expectation_where) =
            ga_expectation.split_for_impl();

        quote! {
            pub fn #ident_expect_method #ga_method_impl(&self) -> #ident_expectation_module::ExpectationBuilder #ga_builder_types
            #ga_method_where
            {
                #ident_expectation_module::ExpectationBuilder::new(parking_lot::MutexGuard::map(self.shared.lock(), |shared| {
                    let exp = #ident_expectation_module::Expectation::#ga_expectation_types::default();

                    shared.#ident_expectation_field.push(Box::new(exp));

                    let exp: &mut dyn gmock::Expectation = &mut **shared.#ident_expectation_field.last_mut().unwrap();

                    unsafe { &mut *(exp as *mut dyn gmock::Expectation as *mut #ident_expectation_module::Expectation #ga_expectation_types) }
                }))
            }
        }
    }
}

impl ToTokens for Handle {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            context,
            methods,
            ga_handle_extra,
        } = self;

        let ContextData { ga_handle, .. } = &**context;

        let (ga_handle_impl, ga_handle_types, ga_handle_where) = ga_handle.split_for_impl();
        let (ga_handle_extra_impl, ga_handle_extra_types, ga_handle_extra_where) =
            ga_handle_extra.split_for_impl();

        let methods = methods.iter().map(|context| self.render_method(context));

        tokens.extend(quote! {
            pub struct Handle #ga_handle_impl #ga_handle_where {
                pub shared: Arc<Mutex<Shared #ga_handle_types>>,
            }

            impl #ga_handle_impl Handle #ga_handle_types #ga_handle_where {
                pub fn checkpoint(&self) {
                    self.shared.lock().checkpoint();
                }
            }

            impl #ga_handle_extra_impl Handle #ga_handle_extra_types #ga_handle_extra_where {
                #( #methods )*
            }

            impl #ga_handle_impl Drop for Handle #ga_handle_types #ga_handle_where {
                fn drop(&mut self) {
                    if !::std::thread::panicking() {
                        self.shared.lock().checkpoint();
                    }
                }
            }
        });
    }
}
