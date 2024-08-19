use quote::{quote, ToTokens};
use syn::ImplItem;

use crate::mock::context::ImplContextData;

use super::context::{Context, ContextData, ImplContext};

pub(crate) struct Mock {
    context: Context,
    impls: Vec<Impl>,
}

struct Impl {
    context: ImplContext,
    items: Vec<ImplItem>,
}

impl Mock {
    pub(crate) fn new(context: Context) -> Self {
        Self {
            context,
            impls: Vec::new(),
        }
    }

    pub(crate) fn add_impl(&mut self, context: ImplContext, items: Vec<ImplItem>) {
        self.impls.push(Impl { context, items });
    }
}

impl ToTokens for Mock {
    #[allow(clippy::too_many_lines)]
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
                    /// Create a new empty mock object.
                    ///
                    /// This method is only generated if the object to mock implements
                    /// the `Default` trait.
                    pub fn new() -> Self {
                        #[allow(clippy::default_trait_access)]
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

            let (ga_impl, _ga_types, ga_where) = ga_impl_mock.split_for_impl();

            quote! {
                impl #ga_impl #trait_ Mock #ga_mock_types #ga_where {
                    #( #items )*
                }
            }
        });

        tokens.extend(quote! {
            /// Mocked version of the type the [`mock!`](crate::mock) macro was executed on.
            #[must_use]
            pub struct Mock #ga_mock_impl #ga_mock_where {
                /// The state is the object that the [`mock!`](crate::mock) macro was executed on.
                /// It is used to execute actual calls to the mocked version of the different methods
                /// of the object.
                pub state: #ident_state #ga_state_types,

                /// Shared state that is used across the different helper objects of one mocked object.
                pub (super) shared: Arc<Mutex<Shared #ga_mock_types>>,

                /// Handle of the current mock object.
                pub (super) handle: Option<Handle #ga_handle_types>
            }

            impl #ga_mock_impl Mock #ga_mock_types #ga_mock_where {
                /// Create a new [`Mock`] instance from the passed `state` object.
                pub fn from_state(state: #ident_state #ga_state_types) -> Self {
                    let handle = Handle::new();
                    let shared = handle.shared.clone();

                    Self {
                        state,
                        shared,
                        handle: Some(handle),
                    }
                }

                /// Get a reference to the handle of this mock object.
                ///
                /// # Panics
                /// May panic if the current mock object does not contain a handle anymore.
                pub fn mock_handle(&self) -> &Handle #ga_handle_types {
                    if let Some(handle) = &self.handle {
                        handle
                    } else {
                        panic!("The handle of this mock object was already taken!");
                    }
                }

                /// Split the current mock object into its handle and a mock object
                /// without a handle.
                ///
                /// # Panics
                /// May panic if the current mock object does not contain a handle anymore.
                pub fn mock_split(mut self) -> (Handle #ga_handle_types, Self) {
                    let handle = self.mock_take_handle();

                    (handle, self)
                }

                /// Extract the handle from the current mock object.
                ///
                /// # Panics
                /// May panic if the current mock object does not contain a handle anymore.
                pub fn mock_take_handle(&mut self) -> Handle #ga_handle_types {
                    if let Some(handle) = self.handle.take() {
                        handle
                    } else {
                        panic!("The handle of this mock object was already taken!");
                    }
                }

                /// Release the handle of this mock object. See [`release`](Handle::release()) for details.
                ///
                /// # Panics
                /// May panic if the current mock object does not contain a handle anymore.
                pub fn mock_release_handle(&mut self) {
                    self.mock_take_handle().release();
                }
            }

            impl #ga_mock_impl Debug for Mock #ga_mock_types #ga_mock_where {
                fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                    f.debug_struct("Mock")
                        .field("shared", &self.shared)
                        .field("handle", &self.handle)
                        .finish_non_exhaustive()
                }
            }

            #mock_clone_impl
            #mock_default_impl

            #( #impls )*
        });
    }
}
