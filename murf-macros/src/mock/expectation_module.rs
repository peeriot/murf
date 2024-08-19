use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{ImplItemFn, ItemImpl};

use super::{
    context::{MethodContext, MethodContextData},
    expectation::Expectation,
    expectation_builder::ExpectationBuilder,
    parsed::Parsed,
};

pub(crate) struct ExpectationModule {
    pub context: MethodContext,
    pub expectation: Expectation,
    pub expectation_builder: ExpectationBuilder,
}

impl ExpectationModule {
    pub(crate) fn new(
        context: MethodContext,
        parsed: &Parsed,
        impl_: &ItemImpl,
        method: &ImplItemFn,
    ) -> Self {
        let expectation = Expectation::new(context.clone(), impl_);
        let expectation_builder = ExpectationBuilder::new(context.clone(), parsed, method);

        Self {
            context,
            expectation,
            expectation_builder,
        }
    }
}

impl ToTokens for ExpectationModule {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            context,
            expectation,
            expectation_builder,
        } = self;

        let MethodContextData {
            trait_,
            ident_method,
            is_associated,
            ident_expectation_module,
            ..
        } = &**context;

        let ident_murf = &context.ident_murf;

        let associated_expectations = if *is_associated {
            Some(quote! {
                pub static EXPECTATIONS: Lazy<Mutex<Vec<Weak<Mutex<Box<dyn #ident_murf :: Expectation + Send + Sync + 'static>>>>>> = Lazy::new(|| Default::default());

                pub fn cleanup_associated_expectations() {
                    EXPECTATIONS.lock().retain_mut(|ex| ex.strong_count() > 0);
                }
            })
        } else {
            None
        };

        let type_ = if let Some(trait_) = trait_ {
            trait_.into_token_stream().to_string()
        } else {
            context.ident_state.to_string()
        };

        let doc = format!("Module that defines the expectation types for [`{type_}::{ident_method}`]({type_}::{ident_method}");

        tokens.extend(quote! {
            #[doc = #doc]
            pub mod #ident_expectation_module {
                use std::marker::PhantomData;
                use std::fmt::{Display, Formatter, Result as FmtResult};

                use #ident_murf :: {
                    Matcher, Times, TimesRange, Sequence, SequenceHandle, InSequence,
                    action::{Action, RepeatableAction, OnetimeAction, RepeatedAction},
                };

                #[allow(clippy::wildcard_imports)]
                use super::*;

                #expectation
                #expectation_builder

                #associated_expectations

                /// Unique ID that represents this type of expectation.
                pub static TYPE_ID: Lazy<usize> = Lazy::new(#ident_murf :: next_type_id);
            }
        });
    }
}
