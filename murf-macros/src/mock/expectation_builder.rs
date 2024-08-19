use std::ops::Not;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ImplItemFn;

use crate::misc::{IterEx, MethodEx, TempLifetimes};

use super::{
    context::{ContextData, MethodContext, MethodContextData},
    parsed::Parsed,
};

pub(crate) struct ExpectationBuilder {
    context: MethodContext,

    must_use: Option<TokenStream>,
}

impl ExpectationBuilder {
    pub(crate) fn new(context: MethodContext, parsed: &Parsed, method: &ImplItemFn) -> Self {
        let must_use = (method.need_default_impl() && !method.has_default_impl() && !parsed.ty.is_extern()).then(|| quote!(#[must_use = "You need to define an action for this expectation because it has no default action!"]));

        ExpectationBuilder { context, must_use }
    }
}

impl ToTokens for ExpectationBuilder {
    #[allow(clippy::too_many_lines)]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { context, must_use } = self;

        let MethodContextData {
            context,
            is_associated,
            ident_expectation_field,
            ga_expectation,
            ga_expectation_builder,
            lts_mock: TempLifetimes(lts_mock),
            args_prepared_lt,
            return_type,
            ..
        } = &**context;

        let ContextData {
            ident_murf,
            trait_send,
            trait_sync,
            ga_handle,
            ..
        } = &****context;

        let trait_send = is_associated
            .then(|| quote!( + Send))
            .or_else(|| trait_send.clone());
        let trait_sync = is_associated
            .then(|| quote!( + Sync))
            .or_else(|| trait_sync.clone());

        let lts_mock = lts_mock.is_empty().not().then(|| quote!(for < #lts_mock >));
        let lt = if *is_associated {
            quote!( + 'static)
        } else {
            quote!( + 'mock)
        };

        let arg_types_prepared_lt = args_prepared_lt.iter().map(|t| &t.ty).parenthesis();

        let drop_handler = if *is_associated {
            quote! {
                let expectation: Box<dyn #ident_murf :: Expectation + Send + Sync + 'static> = Box::new(expectation);
                let expectation = Arc::new(Mutex::new(expectation));
                let weak = Arc::downgrade(&expectation);

                if let Some(local) = #ident_murf :: LocalContext::current().borrow_mut().as_mut() {
                    local.push(*TYPE_ID, weak)
                } else {
                    EXPECTATIONS.lock().push(weak);
                };
            }
        } else {
            quote! {
                let expectation = Box::new(expectation);
            }
        };

        let (_ga_handle_impl, ga_handle_types, _ga_handle_where) = ga_handle.split_for_impl();
        let (_ga_expectation_impl, ga_expectation_types, _ga_expectation_where) =
            ga_expectation.split_for_impl();
        let (
            ga_expectation_builder_impl,
            ga_expectation_builder_types,
            ga_expectation_builder_where,
        ) = ga_expectation_builder.split_for_impl();

        tokens.extend(quote! {
            /// Helper type that is used to set the values of a [`Expectation`] object before it is
            /// added to the list of expected calls of a mock object.
            #must_use
            pub struct ExpectationBuilder #ga_expectation_builder_impl #ga_expectation_builder_where {
                handle: &'mock_exp Handle #ga_handle_types,
                expectation: Option<Expectation #ga_expectation_types>,
            }

            impl #ga_expectation_builder_impl ExpectationBuilder #ga_expectation_builder_types #ga_expectation_builder_where {
                /// Create a new [`ExpectationBuilder`] object
                pub fn new(handle: &'mock_exp Handle #ga_handle_types,) -> Self {
                    let mut expectation = Expectation {
                        sequences: InSequence::create_handle().into_iter().collect(),
                        ..Expectation::default()
                    };
                    expectation.times.range = (1..).into();

                    Self {
                        handle,
                        expectation: Some(expectation),
                    }
                }

                /// Add a description to the expectation.
                pub fn description<S: Into<String>>(mut self, value: S) -> Self {
                    self.expectation().description = Some(value.into());

                    self
                }

                /// Add a [`Matcher`] to the expectation.
                ///
                /// Matchers can be used to verify that the arguments of method call matches the expectation.
                pub fn with<M: #lts_mock Matcher<#arg_types_prepared_lt> #trait_send #trait_sync #lt>(mut self, matcher: M) -> Self {
                    self.expectation().matcher = Some(Box::new(matcher));

                    self
                }

                /// Set the sequence the expectation should be executed in.
                pub fn in_sequence(mut self, sequence: &Sequence) -> Self {
                    self.expectation().sequences = vec![ sequence.create_handle() ];

                    self
                }

                /// Add a sequence the expectation should be executed in.
                pub fn add_sequence(mut self, sequence: &Sequence) -> Self {
                    self.expectation().sequences.push(sequence.create_handle());

                    self
                }

                /// Remove the expectation from all sequences.
                pub fn no_sequences(mut self) -> Self {
                    self.expectation().sequences.clear();

                    self
                }

                /// Specify the number of calls for this expectation.
                pub fn times<R: Into<TimesRange>>(mut self, range: R) -> Self {
                    self.expectation().times.range = range.into();

                    self
                }

                /// Specify an action that should be executed once the actual call to the linked method was made.
                ///
                /// This will set `.times(1)` before the action is added. I you want to use
                /// repeatedly executed actions please have a look at [`will_repeatedly`](Self::will_repeatedly).
                pub fn will_once<A>(self, action: A)
                where
                    A: #lts_mock Action<#arg_types_prepared_lt, #return_type> #trait_send #trait_sync #lt,
                {
                    self.times(1).expectation().action = Some(Box::new(OnetimeAction::new(action)));
                }

                /// Specify an action that should be executed each time a call to the linked method was made.
                pub fn will_repeatedly<A>(mut self, action: A)
                where
                    A: #lts_mock Action<#arg_types_prepared_lt, #return_type> #trait_send #trait_sync + Clone #lt,
                {
                    self.expectation().action = Some(Box::new(RepeatedAction::new(action)));
                }

                fn expectation(&mut self) -> &mut Expectation #ga_expectation_types {
                    self.expectation.as_mut().unwrap()
                }
            }

            impl #ga_expectation_builder_impl Debug for ExpectationBuilder #ga_expectation_builder_types #ga_expectation_builder_where {
                fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                    f.debug_struct("ExpectationBuilder")
                        .field("handle", &self.handle)
                        .field("expectation", self.expectation.as_ref().unwrap())
                        .finish()
                }
            }

            impl #ga_expectation_builder_impl Drop for ExpectationBuilder #ga_expectation_builder_types #ga_expectation_builder_where {
                fn drop(&mut self) {
                    if let Some(expectation) = self.expectation.take() {
                        let desc = expectation.to_string();
                        for seq_handle in &expectation.sequences {
                            seq_handle.set_description(desc.clone());

                            if expectation.times.is_ready() {
                                for seq_handle in &expectation.sequences {
                                    seq_handle.set_ready();
                                }
                            }
                        }

                        #drop_handler;

                        self.handle.shared.lock().#ident_expectation_field.push(expectation);
                    }
                }
            }
        });
    }
}
