use std::ops::Deref;
use std::sync::Arc;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Generics, ImplItemMethod, ItemImpl, PatType, Path, ReturnType, Type};

use crate::misc::{
    format_expect_call, format_expect_module, format_expectations_field, GenericsEx, InputsEx,
    ItemImplEx, MethodEx, ReturnTypeEx, TempLifetimes, TypeEx,
};

use super::parsed::Parsed;

/* Context */

#[derive(Clone)]
pub struct Context(Arc<ContextData>);

pub struct ContextData {
    pub ident_module: Ident,
    pub ident_mock: Ident,
    pub ident_handle: Ident,
    pub ident_state: Ident,

    pub ga_mock: Generics,
    pub ga_state: Generics,
    pub ga_handle: Generics,

    pub derive_send: bool,
    pub derive_sync: bool,
    pub derive_clone: bool,

    pub trait_send: Option<TokenStream>,
    pub trait_sync: Option<TokenStream>,
}

impl Context {
    pub fn new(parsed: &Parsed) -> Self {
        let ident = parsed.ty.ident().to_string();
        let ident_mock = format_ident!("{}Mock", ident);
        let ident_handle = format_ident!("{}Handle", ident);

        let ident = ident.to_case(Case::Snake);
        let ident_module = format_ident!("mock_impl_{}", ident);

        let ident_state = parsed.ty.ident().clone();

        let ga_state = parsed.ty.generics().clone();
        let ga_mock = ga_state.clone().add_lifetime("'mock");
        let ga_handle = ga_mock.clone();

        let derive_send = parsed.derive_send;
        let derive_sync = parsed.derive_sync;
        let derive_clone = parsed.ty.derives("Clone");

        let trait_send = derive_send.then(|| quote!(+ Send));
        let trait_sync = derive_sync.then(|| quote!(+ Sync));

        Self(Arc::new(ContextData {
            ident_module,
            ident_mock,
            ident_handle,
            ident_state,

            ga_mock,
            ga_state,
            ga_handle,

            derive_send,
            derive_sync,
            derive_clone,

            trait_send,
            trait_sync,
        }))
    }
}

impl Deref for Context {
    type Target = ContextData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/* ImplContext */

#[derive(Clone)]
pub struct ImplContext(Arc<ImplContextData>);

impl ImplContext {
    pub fn new(context: Context, impl_: &ItemImpl) -> Self {
        let (impl_, lts_temp) = impl_.clone().split_off_temp_lifetimes();

        let ga_impl = impl_.generics.clone();

        Self(Arc::new(ImplContextData {
            context,

            impl_,
            lts_temp,

            ga_impl,
        }))
    }
}

impl Deref for ImplContext {
    type Target = ImplContextData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ImplContextData {
    pub context: Context,

    pub impl_: ItemImpl,
    pub lts_temp: TempLifetimes,

    pub ga_impl: Generics,
}

impl Deref for ImplContextData {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

/* MethodContext */

#[derive(Clone)]
pub struct MethodContext(Arc<MethodContextData>);

impl MethodContext {
    pub fn new(
        context: ImplContext,
        parsed: &Parsed,
        impl_: &ItemImpl,
        method: &ImplItemMethod,
    ) -> Self {
        let is_associated = method.is_associated_fn();

        let (impl_, lts_temp) = impl_.clone().split_off_temp_lifetimes();
        let trait_ = impl_.trait_.as_ref().map(|(_, x, _)| x).cloned();

        let ga_impl_mock = context.ga_impl.clone().add_lifetime("'mock");

        let mut ga_expectation = context.ga_impl.clone();
        if !is_associated {
            ga_expectation = ga_expectation.add_lifetime("'mock");
        };
        let ga_expectation = ga_expectation.remove_lifetimes(&lts_temp);
        let ga_expectation_builder = ga_impl_mock
            .add_lifetime_clauses("'mock")
            .add_lifetime("'mock_exp")
            .remove_lifetimes(&lts_temp);
        let ga_method = context.ga_impl.clone().remove_other(&context.ga_state);

        let args = method
            .sig
            .inputs
            .without_self_arg()
            .cloned()
            .collect::<Vec<_>>();
        let ret = method.sig.output.clone();

        let (_ga_state_impl, ga_state_types, _ga_state_where) = context.ga_state.split_for_impl();

        let type_ = Type::from_ident(context.ident_state.clone());
        let type_ = Type::Verbatim(quote!( #type_ #ga_state_types ));

        let mut lts_mock = lts_temp.clone();

        let args_with_lt = args
            .iter()
            .map(|i| match &*i.ty {
                Type::Reference(t) => {
                    let mut t = t.clone();
                    t.lifetime = Some(lts_mock.generate());

                    Type::Reference(t).replace_default_lifetime(&mut lts_mock)
                }
                t => t.clone().replace_default_lifetime(&mut lts_mock),
            })
            .collect();

        let args_without_lt = args.iter().map(|i| i.ty.clone().make_static()).collect();

        let args_without_self = args
            .iter()
            .map(|i| i.ty.clone().replace_self_type(&type_))
            .collect::<Vec<_>>();

        let return_type = ret.to_action_return_type(&parsed.ty);

        let ident_method = method.sig.ident.clone();
        let ident_expect_method = format_expect_call(&ident_method, trait_.as_ref());
        let ident_expectation_module = format_expect_module(&ident_method, trait_.as_ref());
        let ident_expectation_field = format_expectations_field(&ident_expectation_module);

        Self(Arc::new(MethodContextData {
            context,

            is_associated,

            impl_,
            trait_,

            ga_method,
            ga_expectation,
            ga_expectation_builder,

            args,
            ret,

            lts_temp,
            lts_mock,

            args_with_lt,
            args_without_lt,
            args_without_self,
            return_type,

            ident_method,
            ident_expect_method,
            ident_expectation_module,
            ident_expectation_field,
        }))
    }
}

impl Deref for MethodContext {
    type Target = MethodContextData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct MethodContextData {
    pub context: ImplContext,

    pub is_associated: bool,

    pub trait_: Option<Path>,
    pub impl_: ItemImpl,

    pub ga_method: Generics,
    pub ga_expectation: Generics,
    pub ga_expectation_builder: Generics,

    pub args: Vec<PatType>,
    pub ret: ReturnType,

    pub lts_temp: TempLifetimes,
    pub lts_mock: TempLifetimes,

    pub args_with_lt: Vec<Type>,
    pub args_without_lt: Vec<Type>,
    pub args_without_self: Vec<Type>,
    pub return_type: Type,

    pub ident_method: Ident,
    pub ident_expect_method: Ident,
    pub ident_expectation_module: Ident,
    pub ident_expectation_field: Ident,
}

impl Deref for MethodContextData {
    type Target = ImplContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
