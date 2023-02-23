use std::mem::take;

use convert_case::{Case, Casing};
use lazy_static::lazy_static;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use regex::{Captures, Regex};
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream},
    parse::{Parser, Result as ParseResult},
    parse2,
    punctuated::Punctuated,
    token::{Gt, Lt},
    Attribute, FnArg, GenericParam, Generics, ImplItem, ImplItemMethod, ImplItemType, Item,
    ItemEnum, ItemImpl, ItemStruct, Lifetime, LifetimeDef, PatType, ReturnType, Stmt, Token, Type,
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

        self.generate_mock_impl(impl_);
        self.generate_handle_impl(impl_);
    }

    fn generate_mock_impl(&mut self, impl_: &ItemImpl) {
        let ga = impl_.generics.add_lifetime("'mock");
        let (ga_impl, ga_types, ga_where) = ga.split_for_impl();

        let trait_ = impl_.trait_.as_ref().map(|x| {
            let x = x.1.to_token_stream();

            quote! {
                #x for
            }
        });

        let items = take(&mut self.mock_items);

        self.result.mock.extend(quote! {
            impl #ga_impl #trait_ Mock #ga_types #ga_where {
                #items
            }
        })
    }

    fn generate_handle_impl(&mut self, impl_: &ItemImpl) {
        let ga = impl_.generics.add_lifetime("'mock");
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
        self.generate_handle_method(impl_, method);
        self.generate_expectation_module(parsed, impl_, method);
    }

    fn generate_mock_method(&mut self, impl_: &ItemImpl, mut method: ImplItemMethod) {
        let trait_ = impl_.trait_.as_ref().map(|(_, x, _)| x);
        let module = format_expect_module(&method.sig.ident, trait_);
        let field = format_expectations_field(&module);

        let args = method.sig.inputs.without_self().map(|i| &i.pat);
        let args = quote! { ( #( #args ),* ) };

        let self_ty = &impl_.self_ty;

        let transmute_args = method.sig.inputs.without_self().map(|i| {
            let pat = &i.pat;
            let ty = i.ty.replace_default_lifetime("'mock");

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
            format!(
                "<{} as {}>",
                impl_.self_ty.to_token_stream(),
                t.to_token_stream()
            )
        } else {
            format!("{}", impl_.self_ty.to_token_stream())
        };
        let error = format!(
            "No suitable expectation found for {}::{}",
            error, &method.sig.ident
        );

        let block = quote! {
            #( #transmute_args )*

            let mut shared = self.shared.lock();
            let args = #args;

            let mut msg = String::new();
            let _ = writeln!(msg, #error);
            let _ = writeln!(msg, "Tried the following expectations:");

            for ex in &mut shared.#field {
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
                    drop(shared);

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

    fn generate_handle_method(&mut self, impl_: &ItemImpl, method: &ImplItemMethod) {
        let trait_ = impl_.trait_.as_ref().map(|(_, x, _)| x);

        let ident = format_expect_call(&method.sig.ident, trait_);
        let module = format_expect_module(&method.sig.ident, trait_);
        let field = format_expectations_field(&module);

        let ga = impl_.generics.add_lifetime("'mock").add_lifetime("'_");
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

        let ga = impl_.generics.add_lifetime("'mock");
        let (ga_impl, ga_types, ga_where) = ga.split_for_impl();
        let ga_phantom = ga.make_phantom_data();

        let ga_builder = ga.add_lifetime("'mock_exp");
        let (ga_builder_impl, ga_builder_types, ga_builder_where) = ga_builder.split_for_impl();

        let args = method
            .sig
            .inputs
            .without_self()
            .map(|i| i.ty.replace_default_lifetime("'mock"));
        let args = quote! { ( #( #args ),* ) };

        let mut temp_lt = false;
        let args_with_lt = method.sig.inputs.without_self().map(|i| match &*i.ty {
            Type::Reference(t) => {
                let mut t = t.clone();
                t.lifetime = Some(Lifetime::new("'mock_tmp", Span::call_site()));

                temp_lt = true;

                Type::Reference(t).replace_default_lifetime("'mock")
            }
            t => t.replace_default_lifetime("'mock"),
        });
        let args_with_lt = quote! { ( #( #args_with_lt ),* ) };

        let temp_lt = if temp_lt {
            Some(quote!(for<'mock_tmp>))
        } else {
            None
        };

        let ret = method.sig.output.to_action_return_type(&parsed.object);

        let display = if let Some((_, t, _)) = &impl_.trait_ {
            format!(
                "<{} as {}>",
                impl_.self_ty.to_token_stream(),
                t.to_token_stream()
            )
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

                pub struct Expectation #ga_types #ga_where {
                    pub times: Times,
                    pub description: Option<String>,
                    pub action: Option<Box<dyn #temp_lt RepeatableAction<#args_with_lt, #ret> + 'mock>>,
                    pub matcher: Option<Box<dyn #temp_lt Matcher<#args_with_lt> + 'mock>>,
                    pub sequence: Option<SequenceHandle>,
                    _marker: #ga_phantom,
                }

                impl #ga_impl Default for Expectation #ga_types #ga_where {
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

                impl #ga_impl Expectation #ga_types #ga_where {
                    pub fn matches(&self, args: &#args) -> bool {
                        if let Some(m) = &self.matcher {
                            m.matches(args)
                        } else {
                            true
                        }
                    }
                }

                impl #ga_impl Display for Expectation #ga_types #ga_where {
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
                    guard: MappedMutexGuard<'mock_exp, Expectation #ga_types>,
                }

                impl #ga_builder_impl ExpectationBuilder #ga_builder_types #ga_builder_where {
                    pub fn new(mut guard: MappedMutexGuard<'mock_exp, Expectation #ga_types>) -> Self {
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

                    pub fn with<M: #temp_lt Matcher<#args_with_lt> + 'mock>(mut self, matcher: M) -> Self {
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
                        A: #temp_lt Action<#args_with_lt, #ret> + 'mock,
                    {
                        self.times(1).guard.action = Some(Box::new(OnetimeAction::new(action)));
                    }

                    pub fn will_repeatedly<A>(mut self, action: A)
                    where
                        A: #temp_lt Action<#args_with_lt, #ret> + Clone + 'mock,
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

/* GenericsEx */

trait GenericsEx {
    fn add_lifetime(&self, lt: &str) -> Self;
    fn make_phantom_data(&self) -> TokenStream;
}

impl GenericsEx for Generics {
    fn add_lifetime(&self, lt: &str) -> Self {
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

    fn without_self(&self) -> Self::Iter<'_>;
}

impl InputsEx for Punctuated<FnArg, Token![,]> {
    type Iter<'x> = Box<dyn Iterator<Item = &'x PatType> + 'x>;

    fn without_self(&self) -> Self::Iter<'_> {
        Box::new(self.iter().filter_map(|i| match i {
            FnArg::Receiver(_) => None,
            FnArg::Typed(t) if t.pat.to_token_stream().to_string() == "self" => None,
            FnArg::Typed(t) => Some(t),
        }))
    }
}

/* TypeEx */

trait TypeEx {
    fn replace_default_lifetime(&self, lf: &str) -> Self;
}

impl TypeEx for Type {
    fn replace_default_lifetime(&self, lf: &str) -> Self {
        let code = self.to_token_stream().to_string();
        let code =
            DEFAULT_LIFETIME_RX.replace_all(&code, |c: &Captures| format!("{}{}", lf, &c[1]));
        Self::Verbatim(code.parse().unwrap())
    }
}

lazy_static! {
    static ref DEFAULT_LIFETIME_RX: Regex = Regex::new(r"'_([>, ])").unwrap();
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
            let mut t = t.clone();
            if let Type::Reference(t) = &mut *t {
                t.lifetime = Some(Lifetime::new("'mock", Span::call_site()));
            }

            let ident = ty.ident();
            let (_, ga_types, _) = ty.generics().split_for_impl();
            let ty = quote!(#ident #ga_types).to_string();

            let code = t.to_token_stream().to_string();
            let code = SELF_RETURN_TYPE
                .replace_all(&code, |c: &Captures| format!("{}{}{}", &c[1], &ty, &c[2]));

            Type::Verbatim(code.parse().unwrap())
        } else {
            Type::Verbatim(quote!(()))
        }
    }
}

lazy_static! {
    static ref SELF_RETURN_TYPE: Regex = Regex::new(r"(^|[^A-Za-z])Self([^A-Za-z]|$)").unwrap();
}
