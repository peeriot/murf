use std::ops::Deref;
use std::sync::Arc;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    FnArg, Generics, ImplItem, ImplItemFn, ItemImpl, Lifetime, Pat, PatType, Path, ReturnType, Type,
};

use crate::misc::{
    format_expect_call, format_expect_module, format_expectations_field, AttribsEx, GenericsEx,
    ItemImplEx, LifetimeReplaceMode, MethodEx, ReturnTypeEx, TempLifetimes, TypeEx,
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
    pub derive_default: bool,

    pub trait_send: Option<TokenStream>,
    pub trait_sync: Option<TokenStream>,

    pub extern_mock_lifetime: bool,
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

        let (_ga_mock_impl, ga_mock_types, _ga_mock_where) = ga_mock.split_for_impl();
        let type_mock = Type::Verbatim(quote!( Mock #ga_mock_types ));

        let mut changed = false;
        let ga_handle = ga_mock.clone().replace_self_type(&type_mock, &mut changed);

        let derive_send = parsed.derive_send;
        let derive_sync = parsed.derive_sync;
        let derive_clone = parsed.ty.derives("Clone");
        let derive_default = parsed.ty.derives("Default");

        let trait_send = derive_send.then(|| quote!(+ Send));
        let trait_sync = derive_sync.then(|| quote!(+ Sync));

        let extern_mock_lifetime = ga_state.lifetimes().any(|lt| lt.lifetime.ident == "mock");

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
            derive_default,

            trait_send,
            trait_sync,

            extern_mock_lifetime,
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
        let ga_impl = impl_.generics.clone();
        let (impl_, lts_temp) = impl_.clone().split_off_temp_lifetimes();

        let need_static_lt = impl_.items.iter().any(|i| {
            if let ImplItem::Fn(f) = i {
                f.is_associated_fn()
                    && matches!(&f.sig.output, ReturnType::Type(_, t) if t.contains_self_type())
            } else {
                false
            }
        });

        let mut ga_impl_mock = ga_impl.clone().add_lifetime("'mock");
        if need_static_lt {
            ga_impl_mock
                .get_lifetime_mut("'mock")
                .unwrap()
                .bounds
                .push(Lifetime::new("'static", Span::call_site()));
        }

        let trait_ = impl_.trait_.as_ref().map(|(_, p, _)| p).cloned();

        Self(Arc::new(ImplContextData {
            context,

            need_static_lt,

            impl_,
            trait_,

            lts_temp,

            ga_impl,
            ga_impl_mock,
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

    pub need_static_lt: bool,

    pub impl_: ItemImpl,
    pub trait_: Option<Path>,

    pub lts_temp: TempLifetimes,

    pub ga_impl: Generics,
    pub ga_impl_mock: Generics,
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
    pub fn new(context: ImplContext, impl_: &ItemImpl, method: &ImplItemFn) -> Self {
        let is_associated = method.is_associated_fn();
        let no_default_impl = method.has_gmock_attr("no_default_impl");

        let (impl_, lts_temp) = impl_.clone().split_off_temp_lifetimes();
        let trait_ = impl_.trait_.as_ref().map(|(_, x, _)| x).cloned();

        let args = method.sig.inputs.iter().cloned().collect::<Vec<_>>();
        let ret = method.sig.output.clone();

        let (_ga_mock_impl, ga_mock_types, _ga_mock_where) = context.ga_mock.split_for_impl();
        let type_mock = Type::Verbatim(quote!( Mock #ga_mock_types ));

        let mut has_self_arg = false;
        let mut lts_mock = lts_temp.clone();

        let args_prepared = args
            .iter()
            .map(|arg| match arg {
                FnArg::Receiver(t) => PatType {
                    attrs: t.attrs.clone(),
                    pat: Box::new(Pat::Verbatim(quote!(this))),
                    colon_token: Default::default(),
                    ty: Box::new(
                        t.ty.clone()
                            .replace_self_type(&type_mock, &mut has_self_arg),
                    ),
                },
                FnArg::Typed(t) => PatType {
                    attrs: t.attrs.clone(),
                    pat: t.pat.clone(),
                    colon_token: t.colon_token,
                    ty: Box::new(
                        t.ty.clone()
                            .replace_self_type(&type_mock, &mut has_self_arg),
                    ),
                },
            })
            .collect::<Vec<_>>();
        let args_prepared_lt = args_prepared
            .iter()
            .cloned()
            .map(|mut t| {
                t.ty = Box::new(
                    t.ty.clone()
                        .replace_default_lifetime(LifetimeReplaceMode::Temp(&mut lts_mock)),
                );

                t
            })
            .collect::<Vec<_>>();

        let mut has_self_ret = false;
        let return_type = ret.to_action_return_type(&type_mock, &mut has_self_ret);

        let type_signature = args_prepared
            .iter()
            .map(|t| t.ty.deref().clone())
            .chain(Some(return_type.clone()))
            .map(TypeEx::make_static)
            .collect();

        let ident_method = method.sig.ident.clone();
        let ident_expect_method = format_expect_call(&ident_method, trait_.as_ref());
        let ident_expectation_module = format_expect_module(&ident_method, trait_.as_ref());
        let ident_expectation_field = format_expectations_field(&ident_expectation_module);

        let mut ga_expectation = context
            .ga_impl
            .clone()
            .merge(&method.sig.generics)
            .replace_self_type(&type_mock, &mut has_self_arg);
        let ga_method = ga_expectation.clone().remove_other(&context.ga_state);

        if !is_associated || has_self_arg || has_self_ret {
            ga_expectation = ga_expectation.add_lifetime("'mock");
            if is_associated && has_self_ret {
                ga_expectation
                    .get_lifetime_mut("'mock")
                    .unwrap()
                    .bounds
                    .push(Lifetime::new("'static", Span::call_site()))
            }
        };
        let ga_expectation = ga_expectation.remove_lifetimes(&lts_temp);

        let mut ga_expectation_builder = ga_expectation
            .clone()
            .add_lifetime_bounds("'mock")
            .add_lifetime("'mock")
            .add_lifetime("'mock_exp")
            .remove_lifetimes(&lts_temp);
        if is_associated && has_self_ret {
            ga_expectation_builder
                .get_lifetime_mut("'mock")
                .unwrap()
                .bounds
                .push(Lifetime::new("'static", Span::call_site()))
        }

        Self(Arc::new(MethodContextData {
            context,

            is_associated,
            no_default_impl,

            impl_,
            trait_,

            ga_method,
            ga_expectation,
            ga_expectation_builder,

            args,
            ret,

            lts_temp,
            lts_mock,

            args_prepared,
            args_prepared_lt,
            return_type,
            type_signature,

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
    pub no_default_impl: bool,

    pub trait_: Option<Path>,
    pub impl_: ItemImpl,

    pub ga_method: Generics,
    pub ga_expectation: Generics,
    pub ga_expectation_builder: Generics,

    pub args: Vec<FnArg>,
    pub ret: ReturnType,

    pub lts_temp: TempLifetimes,
    pub lts_mock: TempLifetimes,

    pub args_prepared: Vec<PatType>,
    pub args_prepared_lt: Vec<PatType>,
    pub return_type: Type,
    pub type_signature: Vec<Type>,

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
