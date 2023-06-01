use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{ImplItemMethod, ItemImpl};

use super::{
    context::{MethodContext, MethodContextData},
    expectation::Expectation,
    expectation_builder::ExpectationBuilder,
    parsed::Parsed,
};

pub struct ExpectationModule {
    pub context: MethodContext,
    pub expectation: Expectation,
    pub expectation_builder: ExpectationBuilder,
}

impl ExpectationModule {
    pub fn new(
        context: MethodContext,
        parsed: &Parsed,
        impl_: &ItemImpl,
        method: &ImplItemMethod,
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
            ident_expectation_module,
            ..
        } = &**context;

        tokens.extend(quote! {
            mod #ident_expectation_module {
                use std::marker::PhantomData;
                use std::fmt::{Display, Formatter, Result as FmtResult};

                use gmock::{
                    Matcher, Times, TimesRange, Sequence, SequenceHandle, InSequence,
                    action::{Action, RepeatableAction, OnetimeAction, RepeatedAction},
                };
                use parking_lot::MappedMutexGuard;

                use super::*;

                #expectation
                #expectation_builder
            }
        });
    }
}
