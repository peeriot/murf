use std::mem::take;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse::{Parser, Result as ParseResult},
    parse2,
    token::{Gt, Lt},
    Attribute, FnArg, GenericParam, Generics, ImplItem, ImplItemMethod, ImplItemType, Item,
    ItemEnum, ItemImpl, ItemStruct, Lifetime, LifetimeDef, Meta, NestedMeta, ReturnType, Stmt,
    Type,
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

        for i in &self.parsed.impls {
            i.to_tokens(tokens);
        }

        let ident = self.parsed.object.ident();
        let ga = self.parsed.object.generics();
        let (ga_impl, ga_types, ga_where) = ga.split_for_impl();

        let ga_mock = ga.add_lifetime("'mock");
        let (ga_mock_impl, ga_mock_types, ga_mock_where) = ga_mock.split_for_impl();
        let ga_mock_phantom = ga_mock.make_phantom_data();

        let expectation_err = format!("Mocked object '{}' has unfulfilled expectations", ident);
        let module = format_ident!("mock_impl_{}", ident.to_string());

        let method_mock = if self.parsed.derives("Default") {
            Some(quote! {
                pub fn mock<'mock>() -> (
                    #module::Handle #ga_mock_types,
                    #module::Mock #ga_mock_types
                ) {
                    Self::default().into_mock()
                }
            })
        } else {
            None
        };

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

        tokens.extend(quote! {
            impl #ga_impl #ident #ga_types #ga_where {
                #method_mock

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

            #[allow(non_snake_case)]
            mod #module {
                use std::sync::Arc;
                use std::fmt::Write;
                use std::marker::PhantomData;

                use parking_lot::{Mutex, MappedMutexGuard, RawMutex};

                use super::*;

                /* Mock */

                pub struct Mock #ga_mock_types #ga_mock_where {
                    pub state: #ident #ga_types,
                    pub shared: Arc<Mutex<Shared #ga_mock_impl >>,
                }

                /* Handle */

                pub struct Handle #ga_mock_types #ga_mock_where {
                    pub shared: Arc<Mutex<Shared #ga_mock_impl >>,
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

                #[derive(Default)]
                #[allow(non_snake_case)]
                pub struct Shared #ga_mock_types #ga_mock_where {
                    #( #expectation_field_defs, )*
                    _marker: #ga_mock_phantom,
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
        println!("\nmock!:\n{:#}\n", tokens);
    }
}

/* Parsed */

struct Parsed {
    object: ObjectToMock,
    impls: Vec<ItemImpl>,
}

impl Parsed {
    fn derives(&self, name: &str) -> bool {
        self.object.attrs().iter().any(|attr| {
            if let Ok(Meta::List(ml)) = attr.parse_meta() {
                let i = ml.path.get_ident();
                if i.map_or(false, |i| *i == "derive") {
                    ml.nested.iter().any(|nm| {
                        if let NestedMeta::Meta(m) = nm {
                            let i = m.path().get_ident();
                            i.map_or(false, |i| *i == name)
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
            generator.generate_impl(i);
        }

        Self::prepare_parsed(&mut parsed);

        MockableObject {
            parsed,
            generated: generator.result,
        }
    }

    fn generate_impl(&mut self, impl_: &ItemImpl) {
        for item in &impl_.items {
            self.generate_item(impl_, item);
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

    fn generate_item(&mut self, impl_: &ItemImpl, item: &ImplItem) {
        #[allow(clippy::single_match)]
        match item {
            ImplItem::Type(type_) => self.generate_type(type_),
            ImplItem::Method(method) => self.generate_method(impl_, method),
            _ => (),
        }
    }

    fn generate_type(&mut self, type_: &ImplItemType) {
        self.mock_items.extend(type_.to_token_stream());
    }

    fn generate_method(&mut self, impl_: &ItemImpl, method: &ImplItemMethod) {
        self.generate_mock_method(impl_, method.clone());
        self.generate_handle_method(impl_, method);
        self.generate_expectation_module(impl_, method);
    }

    fn generate_mock_method(&mut self, impl_: &ItemImpl, mut method: ImplItemMethod) {
        let trait_ = impl_.trait_.as_ref().map(|(_, x, _)| x);
        let module = format_expect_module(&method.sig.ident, trait_);
        let field = format_expectations_field(&module);

        let args = method.sig.inputs.iter().filter_map(|i| match i {
            FnArg::Receiver(_) => None,
            FnArg::Typed(t) => Some(&t.pat),
        });
        let args = quote! { ( #( #args ),* ) };

        let default_args = method.sig.inputs.iter().map(|i| match i {
            FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_some() => {
                quote!(&mut self.state)
            }
            FnArg::Receiver(r) if r.reference.is_some() => quote!(&self.state),
            FnArg::Receiver(_) => quote!(self.state),
            FnArg::Typed(t) => t.pat.to_token_stream(),
        });
        let default_args = quote!( #( #default_args ),* );

        let default_action = if let Some(t) = trait_ {
            let method = &method.sig.ident;

            quote!(#t::#method)
        } else {
            let t = &impl_.self_ty;
            let method = &method.sig.ident;

            quote!(#t::#method)
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

                if let Some(action) = &mut ex.action {
                    return action.exec(args);
                } else {
                    let #args = args;

                    return #default_action(#default_args);
                };
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
            #[allow(non_snake_case)]
            pub fn #ident(&self) -> #module::ExpectationBuilder #ga_types {
                #module::ExpectationBuilder::new(parking_lot::MutexGuard::map(self.shared.lock(), |shared| {
                    let exp = #module::Expectation::default();

                    shared.#field.push(exp);

                    shared.#field.last_mut().unwrap()
                }))
            }
        })
    }

    fn generate_expectation_module(&mut self, impl_: &ItemImpl, method: &ImplItemMethod) {
        let module =
            format_expect_module(&method.sig.ident, impl_.trait_.as_ref().map(|(_, x, _)| x));

        let ga = impl_.generics.add_lifetime("'mock");
        let (ga_impl, ga_types, ga_where) = ga.split_for_impl();

        let ga_builder = ga.add_lifetime("'mock_exp");
        let (ga_builder_impl, ga_builder_types, ga_builder_where) = ga_builder.split_for_impl();

        let args = method.sig.inputs.iter().filter_map(|i| match i {
            FnArg::Receiver(_) => None,
            FnArg::Typed(t) => Some(&t.ty),
        });
        let args = quote! { ( #( #args ),* ) };

        let ret = match &method.sig.output {
            ReturnType::Default => quote! { () },
            ReturnType::Type(_, t) => {
                let mut t = t.clone();
                if let Type::Reference(t) = &mut *t {
                    t.lifetime = Some(Lifetime::new("'mock", Span::call_site()));
                }

                quote! { #t }
            }
        };

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

        let must_use = if Self::need_default_impl(method) && !Self::has_default_impl(method) {
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
            #[allow(non_snake_case)]
            mod #module {
                use std::marker::PhantomData;
                use std::fmt::{Display, Formatter, Result as FmtResult};

                use gmock::{Matcher, Times, TimesRange, Sequence, SequenceHandle, InSequence, action::{Action, RepeatableAction, OnetimeAction, RepeatedAction}};
                use parking_lot::MappedMutexGuard;

                use super::*;

                /* Expectation */

                #[derive(Default)]
                pub struct Expectation #ga_types #ga_where {
                    pub times: Times,
                    pub description: Option<String>,
                    pub action: Option<Box<dyn RepeatableAction<#args, #ret> + 'mock>>,
                    pub matcher: Option<Box<dyn Matcher<#args> + 'mock>>,
                    pub sequence: Option<SequenceHandle>,
                    _marker: PhantomData<&'mock ()>,
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

                    pub fn with<M: Matcher<#args> + 'mock>(mut self, matcher: M) -> Self {
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
                        A: Action<#args, #ret> + 'mock,
                    {
                        self.times(1).guard.action = Some(Box::new(OnetimeAction::new(action)));
                    }

                    pub fn will_repeatedly<A>(mut self, action: A)
                    where
                        A: Action<#args, #ret> + Clone + 'mock,
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

impl Parse for Parsed {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let object = input.parse()?;

        let mut impls = Vec::new();
        while !input.is_empty() {
            impls.push(input.parse()?);
        }

        Ok(Self { object, impls })
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
}

impl ObjectToMock {
    fn ident(&self) -> TokenStream {
        match self {
            Self::Enum(o) => o.ident.to_token_stream(),
            Self::Struct(o) => o.ident.to_token_stream(),
        }
    }

    fn attrs(&self) -> &[Attribute] {
        match self {
            Self::Enum(o) => &o.attrs,
            Self::Struct(o) => &o.attrs,
        }
    }

    fn generics(&self) -> &Generics {
        match self {
            Self::Enum(o) => &o.generics,
            Self::Struct(o) => &o.generics,
        }
    }
}

impl Parse for ObjectToMock {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        match input.parse::<Item>()? {
            Item::Enum(o) => Ok(ObjectToMock::Enum(o)),
            Item::Struct(o) => Ok(ObjectToMock::Struct(o)),
            _ => Err(input.error("Expected either a struct or a enum definition!")),
        }
    }
}

impl ToTokens for ObjectToMock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Enum(o) => o.to_tokens(tokens),
            Self::Struct(o) => o.to_tokens(tokens),
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
            ret.lt_token = Some(Lt {
                spans: [Span::call_site()],
            });
        }

        ret.params.insert(
            0,
            GenericParam::Lifetime(LifetimeDef::new(Lifetime::new(lt, Span::call_site()))),
        );

        if ret.gt_token.is_none() {
            ret.gt_token = Some(Gt {
                spans: [Span::call_site()],
            });
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
