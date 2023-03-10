use std::cell::UnsafeCell;
use std::mem::{take, transmute};

use convert_case::{Case, Casing};
use lazy_static::lazy_static;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use regex::{Captures, Regex};
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream, Parser, Result as ParseResult},
    parse2,
    punctuated::Punctuated,
    token::{Comma, Gt, Lt},
    Attribute, FnArg, GenericArgument, GenericParam, Generics, ImplItem, ImplItemMethod,
    ImplItemType, Item, ItemEnum, ItemImpl, ItemStruct, Lifetime, LifetimeDef, Meta, NestedMeta,
    PatType, Path, PathArguments, PathSegment, ReturnType, Stmt, Token, Type, TypePath,
    WherePredicate,
};

use crate::misc::{format_expect_call, format_expect_module, format_expectations_field};

pub fn exec(input: TokenStream) -> TokenStream {
    match parse2::<MockableObject>(input) {
        Ok(obj) => obj.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }
}

/* MockableObject */

struct MockableObject {
    parsed: Parsed,
    generated: Generated,
}

impl Parse for MockableObject {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        Ok(Generator::generate(input.parse::<Parsed>()?))
    }
}

impl ToTokens for MockableObject {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.parsed.object.to_tokens(tokens);

        if !self.parsed.object.is_extern() {
            for i in &self.parsed.impls {
                i.to_tokens(tokens);
            }
        }

        let ga = self.parsed.object.generics();
        let (ga_impl, ga_types, ga_where) = ga.split_for_impl();

        let ga_mock = ga.add_lifetime("'mock");
        let (ga_mock_impl, ga_mock_types, ga_mock_where) = ga_mock.split_for_impl();
        let ga_mock_phantom = ga_mock.make_phantom_data();

        let mock_default_clone_impl = if self.parsed.object.derives("Clone") {
            Some(quote! {
                impl #ga_mock_impl Clone for Mock #ga_mock_types #ga_mock_where {
                    fn clone(&self) -> Self {
                        Self {
                            state: self.state.clone(),
                            shared: self.shared.clone(),
                        }
                    }
                }
            })
        } else {
            None
        };

        let ga_mock_tmp = ga_mock.add_lifetime("'mock_tmp");
        let (ga_mock_tmp_impl, _, _) = ga_mock_tmp.split_for_impl();

        let ident = self.parsed.object.ident();
        let state = quote!(#ident #ga_types);
        let shared = quote!(Arc<Mutex<Shared #ga_mock_types>>);
        let mock_ident = format_ident!("{}Mock", ident);
        let handle_ident = format_ident!("{}Handle", ident);

        let expectation_err = format!("Mocked object '{ident}' has unfulfilled expectations");
        let module = format_ident!("mock_impl_{}", ident.to_string().to_case(Case::Snake));

        let mock = &self.generated.mock;
        let handle = &self.generated.handle;
        let expectation_modules = &self.generated.expectation_modules;
        let expectation_fields = self
            .generated
            .expectations
            .iter()
            .map(|ident| {
                let field = format_expectations_field(ident);

                quote! {
                    #field
                }
            })
            .collect::<Vec<_>>();
        let expectation_field_defs = self.generated.expectations.iter().map(|ident| {
            let field = format_expectations_field(ident);

            quote! {
                #field: Vec<#ident::Expectation #ga_mock_types>
            }
        });
        let expectation_field_ctor = self.generated.expectations.iter().map(|ident| {
            let field = format_expectations_field(ident);

            quote! {
                #field: Vec::new()
            }
        });

        tokens.extend(quote! {
            impl #ga_impl #state #ga_where {
                pub fn mock<'mock>() -> (
                    #module::Handle #ga_mock_types,
                    #module::Mock #ga_mock_types
                )
                where
                    Self: Default,
                {
                    Self::default().into_mock()
                }

                pub fn into_mock<'mock>(self) -> (
                    #module::Handle #ga_mock_types,
                    #module::Mock #ga_mock_types
                ) {
                    use std::sync::Arc;
                    use parking_lot::Mutex;

                    let shared = Arc::new(Mutex::new(#module::Shared::default()));
                    let handle = #module::Handle {
                        shared: shared.clone()
                    };
                    let mock = #module::Mock {
                        state: self,
                        shared,
                    };

                    (handle, mock)
                }
            }

            pub use #module::Mock as #mock_ident;
            pub use #module::Handle as #handle_ident;

            #[allow(unused_parens)]
            mod #module {
                use std::sync::Arc;
                use std::fmt::Write;
                use std::marker::PhantomData;
                use std::mem::transmute;
                use std::pin::Pin;

                use gmock::{FromState, IntoState};
                use parking_lot::Mutex;

                use super::*;

                /* Mock */

                pub struct Mock #ga_mock_impl #ga_mock_where {
                    pub state: #state,
                    pub shared: #shared,
                }

                #mock_default_clone_impl

                /* IntoState */

                impl #ga_mock_impl IntoState for Mock #ga_mock_types #ga_mock_where {
                    type State = #state;

                    fn into_state(self) -> Self::State {
                        self.state
                    }
                }

                impl #ga_mock_tmp_impl IntoState for &'mock_tmp Mock #ga_mock_types #ga_mock_where {
                    type State = &'mock_tmp #state;

                    fn into_state(self) -> Self::State {
                        &self.state
                    }
                }

                impl #ga_mock_tmp_impl IntoState for &'mock_tmp mut Mock #ga_mock_types #ga_mock_where {
                    type State = &'mock_tmp mut #state;

                    fn into_state(self) -> Self::State {
                        &mut self.state
                    }
                }

                impl #ga_mock_tmp_impl IntoState for Pin<&'mock_tmp mut Mock #ga_mock_types> #ga_mock_where {
                    type State = Pin<&'mock_tmp mut #state>;

                    fn into_state(self) -> Self::State {
                        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().state) }
                    }
                }

                /* FromState */

                impl #ga_mock_impl FromState<#state, #shared> for Mock #ga_mock_types #ga_mock_where {
                    fn from_state(state: #state, shared: #shared) -> Self {
                        Self {
                            state,
                            shared,
                        }
                    }
                }

                impl #ga_mock_impl FromState<Box<#state>, #shared> for Box<Mock #ga_mock_types> #ga_mock_where {
                    fn from_state(state: Box<#state>, shared: #shared) -> Self {
                        Box::new(Mock {
                            state: *state,
                            shared,
                        })
                    }
                }

                /* Handle */

                pub struct Handle #ga_mock_impl #ga_mock_where {
                    pub shared: #shared,
                }

                impl #ga_mock_impl Handle #ga_mock_types #ga_mock_where {
                    pub fn checkpoint(&self) {
                        self.shared.lock().checkpoint();
                    }
                }

                impl #ga_mock_impl Drop for Handle #ga_mock_types #ga_mock_where {
                    fn drop(&mut self) {
                        if !::std::thread::panicking() {
                            self.shared.lock().checkpoint();
                        }
                    }
                }

                /* Shared */

                pub struct Shared #ga_mock_types #ga_mock_where {
                    #( #expectation_field_defs, )*
                    _marker: #ga_mock_phantom,
                }

                impl #ga_mock_impl Default for Shared #ga_mock_types #ga_mock_where {
                    fn default() -> Self {
                        Self {
                            #( #expectation_field_ctor, )*
                            _marker: PhantomData,
                        }
                    }
                }

                impl #ga_mock_impl Shared #ga_mock_types #ga_mock_where {
                    pub(super) fn checkpoint(&self) {
                        let mut raise = false;

                        #(

                            for ex in &self.#expectation_fields {
                                if !ex.times.is_ready() {
                                    if !raise {
                                        println!();
                                        println!(#expectation_err);
                                        raise = true;
                                    }

                                    println!("- {}", &ex);
                                }
                            }

                        )*

                        if raise {
                            println!();
                            panic!(#expectation_err);
                        }
                    }
                }

                #mock
                #handle
                #expectation_modules
            }
        });

        #[cfg(feature = "debug")]
        println!("\nmock!:\n{tokens:#}\n");
    }
}

/* Parsed */

struct Parsed {
    object: ObjectToMock,
    impls: Vec<ItemImpl>,
}

impl Parse for Parsed {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut object = input.parse::<ObjectToMock>()?;

        let mut impls = Vec::new();
        while !input.is_empty() {
            let impl_ = input.parse::<ItemImpl>()?;

            let ident = match &*impl_.self_ty {
                Type::Path(p) if p.qself.is_none() && p.path.leading_colon.is_none() && p.path.segments.len() == 1 => &p.path.segments.last().unwrap().ident,
                _ => return Err(input.error("Expected trait implementation for a simple type that is in the scope of the current module!")),
            };

            if object.is_unknown() {
                object = ObjectToMock::Extern {
                    ident: ident.clone(),
                    generics: impl_.generics.clone(),
                };
            } else if object.ident() != ident {
                return Err(input.error("Implementing mock traits for different type in the same mock!{} block is not supported!"));
            }

            impls.push(impl_);
        }

        Ok(Self { object, impls })
    }
}

/* Generator */

#[derive(Default)]
struct Generator {
    result: Generated,
    mock_items: TokenStream,
    handle_items: TokenStream,
}

impl Generator {
    fn generate(mut parsed: Parsed) -> MockableObject {
        let mut generator = Self::default();

        for i in &parsed.impls {
            generator.generate_impl(&parsed, i);
        }

        Self::prepare_parsed(&mut parsed);

        MockableObject {
            parsed,
            generated: generator.result,
        }
    }

    fn generate_impl(&mut self, parsed: &Parsed, impl_: &ItemImpl) {
        for item in &impl_.items {
            self.generate_item(parsed, impl_, item);
        }

        self.generate_mock_impl(&parsed.object, impl_);
        self.generate_handle_impl(&parsed.object);
    }

    fn generate_mock_impl(&mut self, object: &ObjectToMock, impl_: &ItemImpl) {
        let ga = impl_.generics.add_lifetime("'mock");
        let (ga_impl, _ga_types, ga_where) = ga.split_for_impl();

        let ga_mock = object.generics().add_lifetime("'mock");
        let (_ga_mock_impl, ga_mock_types, _ga_mock_where) = ga_mock.split_for_impl();

        let trait_ = impl_.trait_.as_ref().map(|(_, path, _)| {
            let x = path.to_token_stream();

            quote! {
                #x for
            }
        });

        let items = take(&mut self.mock_items);

        self.result.mock.extend(quote! {
            impl #ga_impl #trait_ Mock #ga_mock_types #ga_where {
                #items
            }
        })
    }

    fn generate_handle_impl(&mut self, object: &ObjectToMock) {
        let ga = object.generics().add_lifetime("'mock");
        let (ga_impl, ga_types, ga_where) = ga.split_for_impl();

        let items = take(&mut self.handle_items);

        self.result.handle.extend(quote! {
            impl #ga_impl Handle #ga_types #ga_where {
                #items
            }
        })
    }

    fn generate_item(&mut self, parsed: &Parsed, impl_: &ItemImpl, item: &ImplItem) {
        #[allow(clippy::single_match)]
        match item {
            ImplItem::Type(type_) => self.generate_type(type_),
            ImplItem::Method(method) => self.generate_method(parsed, impl_, method),
            _ => (),
        }
    }

    fn generate_type(&mut self, type_: &ImplItemType) {
        self.mock_items.extend(type_.to_token_stream());
    }

    fn generate_method(&mut self, parsed: &Parsed, impl_: &ItemImpl, method: &ImplItemMethod) {
        self.generate_mock_method(impl_, method.clone());
        self.generate_handle_method(parsed, impl_, method);
        self.generate_expectation_module(parsed, impl_, method);
    }

    fn generate_mock_method(&mut self, impl_: &ItemImpl, mut method: ImplItemMethod) {
        let trait_ = impl_.trait_.as_ref().map(|(_, x, _)| x);
        let module = format_expect_module(&method.sig.ident, trait_);
        let field = format_expectations_field(&module);

        let args = method.sig.inputs.without_self_arg().map(|i| &i.pat);
        let args = quote! { ( #( #args ),* ) };

        let self_ty = &impl_.self_ty;

        let transmute_args = method.sig.inputs.without_self_arg().map(|i| {
            let pat = &i.pat;
            let ty = &i.ty; // TODO .replace_default_lifetime(&mut );

            quote! {
                let #pat: #ty = unsafe { transmute(#pat) };
            }
        });

        let default_args = method.sig.inputs.iter().map(|i| match i {
            FnArg::Receiver(_) => quote!(self.into_state()),
            FnArg::Typed(t) if t.pat.to_token_stream().to_string() == "self" => {
                quote!(self.into_state())
            }
            FnArg::Typed(t) => t.pat.to_token_stream(),
        });
        let default_args = quote!( #( #default_args ),* );

        let default_action = if let Some(t) = trait_ {
            let method = &method.sig.ident;

            quote!(<#self_ty as #t>::#method)
        } else {
            let t = &impl_.self_ty;
            let method = &method.sig.ident;

            quote!(#t::#method)
        };

        let result = if method.sig.output.contains_self_type() {
            quote!(Self::from_state(ret, self.shared.clone()))
        } else {
            quote!(ret)
        };

        let error = if let Some(t) = trait_ {
            format!("<{} as {}>", impl_.self_ty.to_token_stream(), t.to_string())
        } else {
            format!("{}", impl_.self_ty.to_token_stream())
        };
        let error = format!(
            "No suitable expectation found for {}::{}",
            error, &method.sig.ident
        );

        let block = quote! {
            #( #transmute_args )*

            let mut locked = self.shared.lock();
            let args = #args;

            let mut msg = String::new();
            let _ = writeln!(msg, #error);
            let _ = writeln!(msg, "Tried the following expectations:");

            for ex in &mut locked.#field {
                let _ = writeln!(msg, "- {}", ex);
                if !ex.matches(&args) {
                    let _ = writeln!(msg, "    does not match");
                    continue;
                }

                let _ = writeln!(msg, "    matched");
                if ex.times.is_done() {
                    let _ = writeln!(msg, "    but is already done");

                    continue;
                }

                let _ = writeln!(msg, "    is not done yet");
                if let Some(sequence) = &ex.sequence {
                    if !sequence.check() {
                        let _ = writeln!(msg, "    but is not active");

                        continue;
                    }
                }

                ex.times.increment();

                if let Some(sequence) = &ex.sequence {
                    if ex.times.is_ready() {
                        sequence.set_ready();
                    }
                }

                let ret = if let Some(action) = &mut ex.action {
                    action.exec(args)
                } else {
                    drop(locked);

                    let #args = args;

                    #default_action(#default_args)
                };

                return #result;
            }

            println!("{}", msg);

            panic!(#error);
        };

        method.block.stmts = vec![Stmt::Item(Item::Verbatim(block))];

        self.mock_items.extend(method.into_token_stream());
    }

    fn generate_handle_method(
        &mut self,
        parsed: &Parsed,
        impl_: &ItemImpl,
        method: &ImplItemMethod,
    ) {
        let trait_ = impl_.trait_.as_ref().map(|(_, x, _)| x);

        let ident = format_expect_call(&method.sig.ident, trait_);
        let module = format_expect_module(&method.sig.ident, trait_);
        let field = format_expectations_field(&module);

        let ga = parsed
            .object
            .generics()
            .add_lifetime("'mock")
            .add_lifetime("'_");
        let (_, ga_types, _) = ga.split_for_impl();

        self.handle_items.extend(quote! {
            pub fn #ident(&self) -> #module::ExpectationBuilder #ga_types {
                #module::ExpectationBuilder::new(parking_lot::MutexGuard::map(self.shared.lock(), |shared| {
                    let exp = #module::Expectation::default();

                    shared.#field.push(exp);

                    shared.#field.last_mut().unwrap()
                }))
            }
        })
    }

    fn generate_expectation_module(
        &mut self,
        parsed: &Parsed,
        impl_: &ItemImpl,
        method: &ImplItemMethod,
    ) {
        let module =
            format_expect_module(&method.sig.ident, impl_.trait_.as_ref().map(|(_, x, _)| x));

        let (impl_, lts) = impl_.split_off_temp_lifetimes();
        let mut lts_mock = lts.clone();

        let ga = parsed.object.generics();
        let (_ga_impl, ga_types, _ga_where) = ga.split_for_impl();

        let ga_mock = impl_.generics.add_lifetime("'mock");
        let (ga_mock_impl, ga_mock_types, ga_mock_where) = ga_mock.split_for_impl();
        let ga_mock_phantom = ga_mock.make_phantom_data();

        let ga_builder = ga_mock.add_lifetime("'mock_exp");
        let (ga_builder_impl, ga_builder_types, ga_builder_where) = ga_builder.split_for_impl();

        let type_ = Type::from_ident(parsed.object.ident().clone());
        let type_ = Type::Verbatim(quote!( #type_ #ga_types ));

        let args = method
            .sig
            .inputs
            .without_self_arg()
            .map(|i| i.ty.clone().replace_self_type_owned(&type_));
        let args = quote! { ( #( #args ),* ) };

        let args_with_lt = method.sig.inputs.without_self_arg().map(|i| match &*i.ty {
            Type::Reference(t) => {
                let mut t = t.clone();
                t.lifetime = Some(lts_mock.generate());

                Type::Reference(t).replace_default_lifetime_owned(&mut lts_mock)
            }
            t => t.clone().replace_default_lifetime_owned(&mut lts_mock),
        });
        let args_with_lt = quote! { ( #( #args_with_lt ),* ) };

        let lts = lts.lifetimes;
        let lts = if lts.is_empty() {
            None
        } else {
            Some(quote!(< #lts >))
        };

        let lts_mock = lts_mock.lifetimes;
        let lts_mock = if lts_mock.is_empty() {
            None
        } else {
            Some(quote!(for < #lts_mock >))
        };

        let ret = method.sig.output.to_action_return_type(&parsed.object);

        let display = if let Some((_, t, _)) = &impl_.trait_ {
            format!("<{} as {}>", impl_.self_ty.to_token_stream(), t.to_string())
        } else {
            format!("{}", impl_.self_ty.to_token_stream())
        };
        let display = format!("{}::{}", display, &method.sig.ident);

        let must_use = if Self::need_default_impl(method)
            && !Self::has_default_impl(method)
            && !parsed.object.is_extern()
        {
            Some(
                quote!(#[must_use = "You need to define an action for this expectation because it has no default action!"]),
            )
        } else {
            None
        };

        let default_matcher = method
            .sig
            .inputs
            .iter()
            .map(|_| "_")
            .collect::<Vec<_>>()
            .join(", ");

        self.result.expectation_modules.extend(quote! {
            mod #module {
                use std::marker::PhantomData;
                use std::fmt::{Display, Formatter, Result as FmtResult};

                use gmock::{Matcher, Times, TimesRange, Sequence, SequenceHandle, InSequence, action::{Action, RepeatableAction, OnetimeAction, RepeatedAction}};
                use parking_lot::MappedMutexGuard;

                use super::*;

                /* Expectation */

                pub struct Expectation #ga_mock_types #ga_mock_where {
                    pub times: Times,
                    pub description: Option<String>,
                    pub action: Option<Box<dyn #lts_mock RepeatableAction<#args_with_lt, #ret> + 'mock>>,
                    pub matcher: Option<Box<dyn #lts_mock Matcher<#args_with_lt> + 'mock>>,
                    pub sequence: Option<SequenceHandle>,
                    _marker: #ga_mock_phantom,
                }

                impl #ga_mock_impl Default for Expectation #ga_mock_types #ga_mock_where {
                    fn default() -> Self {
                        Self {
                            times: Times::default(),
                            description: None,
                            action: None,
                            matcher: None,
                            sequence: None,
                            _marker: PhantomData,
                        }
                    }
                }

                impl #ga_mock_impl Expectation #ga_mock_types #ga_mock_where {
                    pub fn matches #lts (&self, args: &#args) -> bool {
                        if let Some(m) = &self.matcher {
                            m.matches(args)
                        } else {
                            true
                        }
                    }
                }

                impl #ga_mock_impl Display for Expectation #ga_mock_types #ga_mock_where {
                    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                        write!(f, #display)?;

                        if let Some(m) = &self.matcher {
                            write!(f, "(")?;
                            m.fmt(f)?;
                            write!(f, ")")?;
                        } else {
                            write!(f, #default_matcher)?;
                        }

                        if let Some(d) = &self.description {
                            write!(f, " {}", d)?;
                        }

                        Ok(())
                    }
                }

                /* ExpectationBuilder */

                #must_use
                pub struct ExpectationBuilder #ga_builder_types #ga_builder_where {
                    guard: MappedMutexGuard<'mock_exp, Expectation #ga_mock_types>,
                }

                impl #ga_builder_impl ExpectationBuilder #ga_builder_types #ga_builder_where {
                    pub fn new(mut guard: MappedMutexGuard<'mock_exp, Expectation #ga_mock_types>) -> Self {
                        guard.sequence = InSequence::create_handle();
                        guard.times.range = (1..usize::max_value()).into();

                        Self {
                            guard,
                        }
                    }

                    pub fn description<S: Into<String>>(mut self, value: S) -> Self {
                        self.guard.description = Some(value.into());

                        self
                    }

                    pub fn with<M: #lts_mock Matcher<#args_with_lt> + 'mock>(mut self, matcher: M) -> Self {
                        self.guard.matcher = Some(Box::new(matcher));

                        self
                    }

                    pub fn in_sequence(mut self, sequence: &Sequence) -> Self {
                        self.guard.sequence = Some(sequence.create_handle());

                        self
                    }

                    pub fn times<R: Into<TimesRange>>(mut self, range: R) -> Self {
                        self.guard.times.range = range.into();

                        self
                    }

                    pub fn will_once<A>(self, action: A)
                    where
                        A: #lts_mock Action<#args_with_lt, #ret> + 'mock,
                    {
                        self.times(1).guard.action = Some(Box::new(OnetimeAction::new(action)));
                    }

                    pub fn will_repeatedly<A>(mut self, action: A)
                    where
                        A: #lts_mock Action<#args_with_lt, #ret> + Clone + 'mock,
                    {
                        self.guard.action = Some(Box::new(RepeatedAction::new(action)));
                    }
                }
            }
        });

        self.result.expectations.push(module);
    }

    fn prepare_parsed(parsed: &mut Parsed) {
        for i in &mut parsed.impls {
            for i in &mut i.items {
                if let ImplItem::Method(m) = i {
                    if !Self::has_default_impl(m) {
                        let block = if Self::need_default_impl(m) {
                            quote!({
                                panic!("No default action specified!");
                            })
                        } else {
                            quote!({})
                        };

                        let attr = Parser::parse2(
                            Attribute::parse_outer,
                            quote!(#[allow(unused_variables)]),
                        )
                        .unwrap();

                        m.attrs.extend(attr);
                        m.block.stmts = vec![Stmt::Item(Item::Verbatim(block))];
                    }
                }
            }
        }
    }

    fn has_default_impl(method: &ImplItemMethod) -> bool {
        !matches!(method.block.stmts.last(), Some(Stmt::Item(Item::Verbatim(v))) if v.to_string() == ";")
    }

    fn need_default_impl(method: &ImplItemMethod) -> bool {
        method.sig.output != ReturnType::Default
    }
}

/* Generated */

#[derive(Default)]
struct Generated {
    mock: TokenStream,
    handle: TokenStream,
    expectations: Vec<Ident>,
    expectation_modules: TokenStream,
}

/* ObjectToMock */

enum ObjectToMock {
    Enum(ItemEnum),
    Struct(ItemStruct),
    Extern { ident: Ident, generics: Generics },
    Unknown,
}

impl ObjectToMock {
    fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    fn is_extern(&self) -> bool {
        matches!(self, Self::Extern { .. })
    }

    fn ident(&self) -> &Ident {
        match self {
            Self::Enum(o) => &o.ident,
            Self::Struct(o) => &o.ident,
            Self::Extern { ident, .. } => ident,
            Self::Unknown => panic!("Unknown mock object!"),
        }
    }

    fn generics(&self) -> &Generics {
        match self {
            Self::Enum(o) => &o.generics,
            Self::Struct(o) => &o.generics,
            Self::Extern { generics, .. } => generics,
            Self::Unknown => panic!("Unknown mock object!"),
        }
    }

    fn attributes(&self) -> &[Attribute] {
        match self {
            Self::Enum(o) => &o.attrs,
            Self::Struct(o) => &o.attrs,
            Self::Extern { .. } => &[],
            Self::Unknown => panic!("Unknown mock object!"),
        }
    }

    fn derives(&self, ident: &str) -> bool {
        self.attributes().iter().any(|attr| {
            if let Ok(Meta::List(ml)) = attr.parse_meta() {
                let i = ml.path.get_ident();
                if i.map_or(false, |i| *i == "derive") {
                    ml.nested.iter().any(|nm| {
                        if let NestedMeta::Meta(m) = nm {
                            let i = m.path().get_ident();
                            i.map_or(false, |i| *i == ident)
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            } else {
                false
            }
        })
    }
}

impl Parse for ObjectToMock {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let fork = input.fork();
        let ret = match fork.parse::<Item>()? {
            Item::Enum(o) => Self::Enum(o),
            Item::Struct(o) => Self::Struct(o),
            _ => return Ok(Self::Unknown),
        };

        input.advance_to(&fork);

        Ok(ret)
    }
}

impl ToTokens for ObjectToMock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Enum(o) => o.to_tokens(tokens),
            Self::Struct(o) => o.to_tokens(tokens),
            _ => (),
        }
    }
}

/* TempLifetimes */

#[derive(Default, Debug, Clone)]
pub struct TempLifetimes {
    lifetimes: Punctuated<Lifetime, Comma>,
    count: usize,
}

impl TempLifetimes {
    fn new(lifetimes: Punctuated<Lifetime, Comma>) -> Self {
        Self {
            lifetimes,
            count: 0,
        }
    }

    fn generate(&mut self) -> Lifetime {
        self.count += 1;

        let lt = format!("'gmock_tmp_{}", self.count);
        let lt = Lifetime::new(&lt, Span::call_site());

        self.lifetimes.push(lt.clone());

        lt
    }
}

/* ItemImplEx */

trait ItemImplEx: Sized {
    fn split_off_temp_lifetimes(&self) -> (Self, TempLifetimes);
}

impl ItemImplEx for ItemImpl {
    fn split_off_temp_lifetimes(&self) -> (Self, TempLifetimes) {
        let mut ret = self.clone();
        let mut lts = Punctuated::default();

        let params = take(&mut ret.generics.params);

        for param in params {
            match param {
                GenericParam::Lifetime(lt) if !ret.self_ty.contains_lifetime(&lt.lifetime) => {
                    if let Some(wc) = &mut ret.generics.where_clause {
                        wc.predicates = wc.predicates.iter().filter_map(|p| {
                            if matches!(p, WherePredicate::Lifetime(plt) if plt.lifetime == lt.lifetime) {
                                None
                            } else {
                                Some(p.clone())
                            }
                        }).collect();
                    }

                    lts.push(lt.lifetime);
                }
                param => ret.generics.params.push(param),
            }
        }

        if ret.generics.params.is_empty() {
            ret.generics.lt_token = None;
            ret.generics.gt_token = None;
        }

        if ret
            .generics
            .where_clause
            .as_ref()
            .map(|wc| wc.predicates.is_empty())
            .unwrap_or(false)
        {
            ret.generics.where_clause = None;
        }

        (ret, TempLifetimes::new(lts))
    }
}

/* GenericsEx */

trait GenericsEx {
    fn add_lifetime(&self, lt: &str) -> Self;
    fn make_phantom_data(&self) -> TokenStream;
}

impl GenericsEx for Generics {
    fn add_lifetime(&self, lt: &str) -> Self {
        for x in &self.params {
            if matches!(x, GenericParam::Lifetime(x) if x.lifetime.to_string() == lt) {
                return self.clone();
            }
        }

        let mut ret = self.clone();

        if ret.lt_token.is_none() {
            ret.lt_token = Some(Lt::default());
        }

        ret.params.insert(
            0,
            GenericParam::Lifetime(LifetimeDef::new(Lifetime::new(lt, Span::call_site()))),
        );

        if ret.gt_token.is_none() {
            ret.gt_token = Some(Gt::default());
        }

        ret
    }

    fn make_phantom_data(&self) -> TokenStream {
        let params = self.params.iter().map(|param| match param {
            GenericParam::Lifetime(lt) => {
                let lt = &lt.lifetime;

                quote!(& #lt ())
            }
            GenericParam::Type(ty) => {
                let ident = &ty.ident;

                quote!(#ident)
            }
            GenericParam::Const(ct) => {
                let ident = &ct.ident;

                quote!(#ident)
            }
        });

        quote!(PhantomData<(#( #params ),*)>)
    }
}

/* InputsEx */

trait InputsEx {
    type Iter<'x>: Iterator<Item = &'x PatType> + 'x
    where
        Self: 'x;

    fn without_self_arg(&self) -> Self::Iter<'_>;
}

impl InputsEx for Punctuated<FnArg, Token![,]> {
    type Iter<'x> = Box<dyn Iterator<Item = &'x PatType> + 'x>;

    fn without_self_arg(&self) -> Self::Iter<'_> {
        Box::new(self.iter().filter_map(|i| match i {
            FnArg::Receiver(_) => None,
            FnArg::Typed(t) if t.pat.to_token_stream().to_string() == "self" => None,
            FnArg::Typed(t) => Some(t),
        }))
    }
}

/* TypeEx */

trait TypeEx: Clone {
    fn from_ident(ident: Ident) -> Self;
    fn contains_lifetime(&self, lt: &Lifetime) -> bool;
    fn replace_self_type(&mut self, type_: &Type);
    fn replace_default_lifetime(&mut self, lts: &mut TempLifetimes);

    fn replace_self_type_owned(mut self, type_: &Type) -> Self {
        self.replace_self_type(type_);

        self
    }

    fn replace_default_lifetime_owned(mut self, lts: &mut TempLifetimes) -> Self {
        self.replace_default_lifetime(lts);

        self
    }
}

trait TypeVisitor {
    fn visit_type(&mut self, ty: &UnsafeCell<Type>) -> bool {
        let _ty = ty;

        true
    }

    fn visit_lifetime(&mut self, lt: &UnsafeCell<Lifetime>) -> bool {
        let _lt = lt;

        true
    }

    fn visit(&mut self, ty: &UnsafeCell<Type>) -> bool {
        if !self.visit_type(ty) {
            return false;
        }

        let ty = unsafe { &*ty.get() };

        match ty {
            Type::Path(ty) => {
                for seg in &ty.path.segments {
                    match &seg.arguments {
                        PathArguments::None => (),
                        PathArguments::AngleBracketed(x) => {
                            for arg in &x.args {
                                match arg {
                                    GenericArgument::Type(t) => {
                                        if !self.visit(as_unsafe_cell(t)) {
                                            return false;
                                        }
                                    }
                                    GenericArgument::Lifetime(lt) => {
                                        if !self.visit_lifetime(as_unsafe_cell(lt)) {
                                            return false;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                        PathArguments::Parenthesized(x) => {
                            for t in &x.inputs {
                                if !self.visit(as_unsafe_cell(t)) {
                                    return false;
                                }
                            }

                            match &x.output {
                                ReturnType::Type(_, t) => {
                                    if !self.visit(as_unsafe_cell(t)) {
                                        return false;
                                    }
                                }
                                ReturnType::Default => (),
                            }
                        }
                    }
                }

                true
            }
            Type::Reference(t) => {
                if let Some(lt) = &t.lifetime {
                    if !self.visit_lifetime(as_unsafe_cell(lt)) {
                        return false;
                    }
                }

                if !self.visit(as_unsafe_cell(&t.elem)) {
                    return false;
                }

                true
            }
            Type::Array(t) => self.visit(as_unsafe_cell(&t.elem)),
            Type::Slice(t) => self.visit(as_unsafe_cell(&t.elem)),
            Type::Tuple(t) => {
                for t in &t.elems {
                    if !self.visit(as_unsafe_cell(t)) {
                        return false;
                    }
                }

                true
            }
            _ => true,
        }
    }
}

impl TypeEx for Type {
    fn from_ident(ident: Ident) -> Self {
        let mut path = Path {
            leading_colon: None,
            segments: Punctuated::default(),
        };
        path.segments.push(PathSegment {
            ident,
            arguments: PathArguments::None,
        });

        Self::Path(TypePath { qself: None, path })
    }

    fn contains_lifetime(&self, lt: &Lifetime) -> bool {
        struct Visitor<'a> {
            lt: &'a Lifetime,
            result: bool,
        }

        impl<'a> TypeVisitor for Visitor<'a> {
            fn visit_lifetime(&mut self, lt: &UnsafeCell<Lifetime>) -> bool {
                let lt = unsafe { &*lt.get() };
                self.result = self.lt.ident == lt.ident || self.result;

                !self.result
            }
        }

        let mut visitor = Visitor { lt, result: false };

        visitor.visit(as_unsafe_cell(self));

        visitor.result
    }

    fn replace_self_type(&mut self, type_: &Type) {
        struct Visitor<'a> {
            type_: &'a Type,
        }

        impl<'a> TypeVisitor for Visitor<'a> {
            fn visit_type(&mut self, ty: &UnsafeCell<Type>) -> bool {
                let ty = unsafe { &mut *ty.get() };

                if let Type::Path(t) = ty {
                    if t.path.segments.len() == 1 && t.path.segments[0].ident == "Self" {
                        *ty = self.type_.clone();
                    }
                }

                true
            }
        }

        let mut visitor = Visitor { type_ };

        visitor.visit(as_unsafe_cell(self));
    }

    fn replace_default_lifetime(&mut self, lts: &mut TempLifetimes) {
        struct Visitor<'a> {
            lts: &'a mut TempLifetimes,
        }

        impl<'a> TypeVisitor for Visitor<'a> {
            fn visit_lifetime(&mut self, lt: &UnsafeCell<Lifetime>) -> bool {
                let lt = unsafe { &mut *lt.get() };

                if lt.ident == "_" {
                    *lt = self.lts.generate();
                }

                true
            }
        }

        let mut visitor = Visitor { lts };

        visitor.visit(as_unsafe_cell(self));
    }
}

/* ReturnTypeEx */

trait ReturnTypeEx {
    fn contains_self_type(&self) -> bool;
    fn to_action_return_type(&self, ty: &ObjectToMock) -> Type;
}

impl ReturnTypeEx for ReturnType {
    fn contains_self_type(&self) -> bool {
        SELF_RETURN_TYPE.is_match(&self.to_token_stream().to_string())
    }

    fn to_action_return_type(&self, ty: &ObjectToMock) -> Type {
        if let ReturnType::Type(_, t) = &self {
            let ident = ty.ident();
            let (_, ga_types, _) = ty.generics().split_for_impl();
            let ty = parse2(quote!(#ident #ga_types)).unwrap();

            let mut t = t.clone().replace_self_type_owned(&ty);
            if let Type::Reference(t) = &mut t {
                t.lifetime = Some(Lifetime::new("'mock", Span::call_site()));
            }

            t
        } else {
            Type::Verbatim(quote!(()))
        }
    }
}

lazy_static! {
    static ref SELF_RETURN_TYPE: Regex = Regex::new(r"(^|[^A-Za-z])Self([^A-Za-z]|$)").unwrap();
}

/* PathEx */

trait PathEx {
    fn to_string(&self) -> String;
}

impl PathEx for Path {
    fn to_string(&self) -> String {
        let code = self.to_token_stream().to_string();

        PATH_FORMAT
            .replace_all(&code, |c: &Captures| c[1].to_string())
            .into_owned()
    }
}

lazy_static! {
    static ref PATH_FORMAT: Regex = Regex::new(r"\s*(<|>)\s*").unwrap();
}

/* Misc */

fn as_unsafe_cell<T>(value: &T) -> &UnsafeCell<T> {
    unsafe { transmute(value) }
}
