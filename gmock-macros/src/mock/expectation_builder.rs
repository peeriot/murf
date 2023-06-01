use std::ops::Not;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ImplItemMethod;

use crate::misc::{MethodEx, TempLifetimes};

use super::{
    context::{ContextData, MethodContext, MethodContextData},
    parsed::Parsed,
};

pub struct ExpectationBuilder {
    context: MethodContext,

    must_use: Option<TokenStream>,
}

impl ExpectationBuilder {
    pub fn new(context: MethodContext, parsed: &Parsed, method: &ImplItemMethod) -> Self {
        let must_use = (method.need_default_impl() && !method.has_default_impl() && !parsed.ty.is_extern()).then(|| quote!(#[must_use = "You need to define an action for this expectation because it has no default action!"]));

        ExpectationBuilder { context, must_use }
    }
}

impl ToTokens for ExpectationBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { context, must_use } = self;

        let MethodContextData {
            context,
            ga_expectation,
            ga_expectation_builder,
            lts_mock: TempLifetimes(lts_mock),
            args_with_lt,
            return_type,
            ..
        } = &**context;

        let ContextData {
            trait_send,
            trait_sync,
            ..
        } = &****context;

        let lts_mock = lts_mock.is_empty().not().then(|| quote!(for < #lts_mock >));

        let (_ga_expectation_impl, ga_expectation_types, _ga_expectation_where) =
            ga_expectation.split_for_impl();
        let (
            ga_expectation_builder_impl,
            ga_expectation_builder_types,
            ga_expectation_builder_where,
        ) = ga_expectation_builder.split_for_impl();

        tokens.extend(quote! {
            #must_use
            pub struct ExpectationBuilder #ga_expectation_builder_impl #ga_expectation_builder_where {
                guard: MappedMutexGuard<'mock_exp, Expectation #ga_expectation_types>,
            }

            impl #ga_expectation_builder_impl ExpectationBuilder #ga_expectation_builder_types #ga_expectation_builder_where {
                pub fn new(mut guard: MappedMutexGuard<'mock_exp, Expectation #ga_expectation_types>) -> Self {
                    guard.sequences = InSequence::create_handle().into_iter().collect();
                    guard.times.range = (1..).into();

                    Self {
                        guard,
                    }
                }

                pub fn description<S: Into<String>>(mut self, value: S) -> Self {
                    self.guard.description = Some(value.into());

                    self
                }

                pub fn with<M: #lts_mock Matcher<( #( #args_with_lt ),* )> #trait_send #trait_sync + 'mock>(mut self, matcher: M) -> Self {
                    self.guard.matcher = Some(Box::new(matcher));

                    self
                }

                pub fn in_sequence(mut self, sequence: &Sequence) -> Self {
                    self.guard.sequences = vec![ sequence.create_handle() ];

                    self
                }

                pub fn add_sequence(mut self, sequence: &Sequence) -> Self {
                    self.guard.sequences.push(sequence.create_handle());

                    self
                }

                pub fn no_sequences(mut self) -> Self {
                    self.guard.sequences.clear();

                    self
                }

                pub fn times<R: Into<TimesRange>>(mut self, range: R) -> Self {
                    self.guard.times.range = range.into();

                    self
                }

                pub fn will_once<A>(self, action: A)
                where
                    A: #lts_mock Action<( #( #args_with_lt ),* ), #return_type> #trait_send #trait_sync + 'mock,
                {
                    self.times(1).guard.action = Some(Box::new(OnetimeAction::new(action)));
                }

                pub fn will_repeatedly<A>(mut self, action: A)
                where
                    A: #lts_mock Action<( #( #args_with_lt ),* ), #return_type> #trait_send #trait_sync + Clone + 'mock,
                {
                    self.guard.action = Some(Box::new(RepeatedAction::new(action)));
                }
            }

            impl #ga_expectation_builder_impl Drop for ExpectationBuilder #ga_expectation_builder_types #ga_expectation_builder_where {
                fn drop(&mut self) {
                    for seq_handle in &self.guard.sequences {
                        seq_handle.set_description(self.guard.to_string());
                    }
                }
            }
        });
    }
}
