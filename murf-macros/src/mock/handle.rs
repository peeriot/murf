use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Generics;

use crate::misc::GenericsEx;

use super::context::{Context, ContextData, MethodContext, MethodContextData};

pub(crate) struct Handle {
    context: Context,
    methods: Vec<MethodContext>,
    ga_handle_extra: Generics,
}

impl Handle {
    pub(crate) fn new(context: Context) -> Self {
        let ga_handle_extra = context.ga_handle.clone().add_lifetime_bounds("'mock");

        Self {
            context,
            methods: Vec::new(),
            ga_handle_extra,
        }
    }

    pub(crate) fn add_method(&mut self, context: MethodContext) {
        self.methods.push(context);
    }

    fn render_method(context: &MethodContextData) -> TokenStream {
        let MethodContextData {
            is_associated,
            ident_expect_method,
            ident_expectation_module,
            ga_method,
            ga_expectation,
            ..
        } = context;

        let mut ga_builder = ga_expectation.clone();
        if *is_associated {
            ga_builder = ga_builder.add_lifetime("'mock");
        }
        ga_builder = ga_builder.add_lifetime("'_");

        let (ga_method_impl, _ga_method_types, ga_method_where) = ga_method.split_for_impl();
        let (_ga_builder_impl, ga_builder_types, _ga_builder_where) = ga_builder.split_for_impl();

        quote! {
            pub fn #ident_expect_method #ga_method_impl(&self) -> #ident_expectation_module::ExpectationBuilder #ga_builder_types
            #ga_method_where
            {
                #ident_expectation_module::ExpectationBuilder::new(&self)
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

        let methods = methods.iter().map(|context| Self::render_method(context));

        tokens.extend(quote! {
            pub struct Handle #ga_handle_impl #ga_handle_where {
                pub shared: Arc<Mutex<Shared #ga_handle_types>>,
                pub check_on_drop: bool,
            }

            impl #ga_handle_impl Handle #ga_handle_types #ga_handle_where {
                pub fn checkpoint(&self) {
                    if let Some(mut locked) = self.shared.try_lock() {
                        locked.checkpoint();
                    } else {
                        panic!("Unable to lock handle: Deadlock? Make sure that you do not drop a handle (or call `checkpoint` directly) of the same object inside an action of an expectation.");
                    }
                }

                pub fn mock_handle(&self) -> &Self {
                    self
                }

                pub fn release(mut self) {
                    self.check_on_drop = false;

                    drop(self);
                }
            }

            impl #ga_handle_extra_impl Handle #ga_handle_extra_types #ga_handle_extra_where {
                #( #methods )*
            }

            impl #ga_handle_impl Handle #ga_handle_types #ga_handle_where {
                pub fn new() -> Self {
                    Self {
                        shared: Default::default(),
                        check_on_drop: true,
                    }
                }
            }

            impl #ga_handle_impl Clone for Handle #ga_handle_types #ga_handle_where {
                fn clone(&self) -> Self {
                    Self {
                        shared: self.shared.clone(),
                        check_on_drop: self.check_on_drop,
                    }
                }
            }

            impl #ga_handle_impl Default for Handle #ga_handle_types #ga_handle_where {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl #ga_handle_impl Drop for Handle #ga_handle_types #ga_handle_where {
                fn drop(&mut self) {
                    if self.check_on_drop && !::std::thread::panicking() {
                        self.checkpoint();
                    }
                }
            }
        });
    }
}
