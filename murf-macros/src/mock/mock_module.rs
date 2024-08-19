use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{ImplItem, ImplItemFn, ItemImpl};

use super::{
    context::{Context, ContextData, ImplContext, MethodContext},
    expectation_module::ExpectationModule,
    handle::Handle,
    mock::Mock,
    mock_method::MockMethod,
    mockable::Mockable,
    mockable_default::MockableDefault,
    parsed::Parsed,
    shared::Shared,
};

/* MockModule */

pub(crate) struct MockModule {
    pub context: Context,
    pub mock: Mock,
    pub mockable: Mockable,
    pub mockable_default: MockableDefault,
    pub handle: Handle,
    pub shared: Shared,
    pub expectations: Vec<ExpectationModule>,
}

impl MockModule {
    pub(crate) fn new(parsed: &Parsed) -> Self {
        let context = Context::new(parsed);
        let mock = Mock::new(context.clone());
        let mockable = Mockable::new(context.clone());
        let mockable_default = MockableDefault::new(context.clone());
        let handle = Handle::new(context.clone());
        let shared = Shared::new(context.clone());
        let expectations = Vec::new();

        let mut ret = Self {
            context,
            mock,
            mockable,
            mockable_default,
            handle,
            shared,
            expectations,
        };

        ret.generate(parsed);

        ret
    }

    fn generate(&mut self, parsed: &Parsed) {
        for impl_ in &parsed.impls {
            let context = ImplContext::new(self.context.clone(), impl_);

            self.generate_impl(context, parsed, impl_);
        }
    }

    fn generate_impl(&mut self, context: ImplContext, parsed: &Parsed, impl_: &ItemImpl) {
        let items = impl_
            .items
            .iter()
            .map(|item| self.generate_item(context.clone(), parsed, impl_, item))
            .collect();

        self.mock.add_impl(context, items);
    }

    fn generate_item(
        &mut self,
        context: ImplContext,
        parsed: &Parsed,
        impl_: &ItemImpl,
        item: &ImplItem,
    ) -> ImplItem {
        match item {
            ImplItem::Fn(f) => ImplItem::Fn(self.generate_method(context, parsed, impl_, f)),
            item => item.clone(),
        }
    }

    fn generate_method(
        &mut self,
        context: ImplContext,
        parsed: &Parsed,
        impl_: &ItemImpl,
        method: &ImplItemFn,
    ) -> ImplItemFn {
        let context = MethodContext::new(context, impl_, method);

        let ret = MockMethod::render(&context, method.clone());

        self.handle.add_method(context.clone());
        self.shared.add_expectation(context.clone());
        self.expectations
            .push(ExpectationModule::new(context, parsed, impl_, method));

        ret
    }
}

impl ToTokens for MockModule {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            context,
            mock,
            mockable,
            mockable_default,
            handle,
            shared,
            expectations,
        } = self;

        let ContextData {
            ident_module,
            ident_mock,
            ident_handle,
            ..
        } = &**context;

        tokens.extend(quote! {
            pub use #ident_module::Mock as #ident_mock;
            pub use #ident_module::Handle as #ident_handle;
            pub use #ident_module::{Mockable as _, MockableDefault as _};

            mod #ident_module {
                use std::any::type_name;
                use std::fmt::Write;
                use std::marker::PhantomData;
                use std::mem::take;
                use std::sync::{Arc, Weak};

                use parking_lot::Mutex;
                use murf::{Lazy, Expectation};

                use super::*;

                pub trait IntoState {
                    type State;

                    fn into_state(self) -> Self::State;
                }

                pub trait FromState<TState, TShared> {
                    fn from_state(state: TState, shared: TShared) -> Self;
                }

                #mock
                #mockable
                #mockable_default
                #handle
                #shared

                #( #expectations )*
            }
        });
    }
}
