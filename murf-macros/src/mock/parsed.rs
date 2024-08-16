use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream, Parser, Result as ParseResult},
    parse2,
    punctuated::Punctuated,
    token::{Brace, Comma},
    Attribute, Block, Generics, ImplItem, ImplItemFn, Item, ItemEnum, ItemImpl, ItemStruct, Meta,
    Path, ReturnType, Stmt, TraitItemFn, Type, Visibility,
};

use crate::misc::AttribsEx;

/// Parsed code inside the mock! macro
pub struct Parsed {
    pub ty: TypeToMock,
    pub impls: Vec<ItemImpl>,

    pub derive_sync: bool,
    pub derive_send: bool,
}

impl Parsed {
    fn add_default_impl(impl_: &mut ItemImpl) -> ParseResult<()> {
        for i in &mut impl_.items {
            if let ImplItem::Verbatim(ts) = i {
                let TraitItemFn { mut attrs, sig, .. } = parse2::<TraitItemFn>(ts.clone())?;

                let mut block = Block {
                    brace_token: Brace::default(),
                    stmts: Vec::new(),
                };

                if sig.output != ReturnType::Default {
                    block.stmts = vec![Stmt::Item(Item::Verbatim(quote!(
                        panic!("No default action specified!");
                    )))];
                }

                let attr = quote!(#[allow(unused_variables)]);
                attrs.extend(Parser::parse2(Attribute::parse_outer, attr).unwrap());

                *i = ImplItem::Fn(ImplItemFn {
                    attrs,
                    vis: Visibility::Inherited,
                    defaultness: None,
                    sig,
                    block,
                })
            }
        }

        Ok(())
    }

    fn remove_uneeded_derives(ty: &mut TypeToMock) -> ParseResult<()> {
        let attrs = match ty {
            TypeToMock::Enum(o) => Some(&mut o.attrs),
            TypeToMock::Struct(o) => Some(&mut o.attrs),
            _ => None,
        };

        if let Some(attrs) = attrs {
            for attr in attrs {
                if attr.path().is_ident("derive") {
                    if let Meta::List(ml) = &mut attr.meta {
                        let mut ret = Option::<Punctuated<Path, Comma>>::None;

                        ml.parse_args_with(|p: ParseStream<'_>| {
                            let ml = Punctuated::<Path, Comma>::parse_separated_nonempty(p)?;

                            ret = Some(
                                ml.into_iter()
                                    .filter(|p| !p.is_ident("Send") && !p.is_ident("Sync"))
                                    .collect(),
                            );

                            Ok(())
                        })?;

                        if let Some(ret) = ret {
                            ml.tokens = ret.into_token_stream();
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Parse for Parsed {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut ty = input.parse::<TypeToMock>()?;

        let derive_send = ty.derives("Send");
        let derive_sync = ty.derives("Sync");

        Self::remove_uneeded_derives(&mut ty)?;

        let mut impls = Vec::new();
        while !input.is_empty() {
            let mut impl_ = input.parse::<ItemImpl>()?;

            let ident = match &*impl_.self_ty {
                Type::Path(p) if p.qself.is_none() && p.path.leading_colon.is_none() && p.path.segments.len() == 1 => &p.path.segments.last().unwrap().ident,
                _ => return Err(input.error("Expected trait implementation for a simple type that is in the scope of the current module!")),
            };

            if ty.is_unknown() {
                ty = TypeToMock::Extern {
                    ident: ident.clone(),
                    generics: impl_.generics.clone(),
                };
            } else if ty.ident() != ident {
                return Err(input.error("Implementing mock traits for different type in the same mock!{} block is not supported!"));
            }

            Self::add_default_impl(&mut impl_)?;

            impls.push(impl_);
        }

        Ok(Self {
            ty,
            impls,
            derive_send,
            derive_sync,
        })
    }
}

impl ToTokens for Parsed {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ty.to_tokens(tokens);

        if !self.ty.is_extern() {
            for i in &self.impls {
                i.clone().remove_murf_attrs().to_tokens(tokens);
            }
        }
    }
}

/// Object the mock is implemented for
pub enum TypeToMock {
    Enum(ItemEnum),
    Struct(ItemStruct),
    Extern { ident: Ident, generics: Generics },
    Unknown { ident: Ident, generics: Generics },
}

impl TypeToMock {
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }

    pub fn is_extern(&self) -> bool {
        matches!(self, Self::Extern { .. })
    }

    pub fn ident(&self) -> &Ident {
        match self {
            Self::Enum(o) => &o.ident,
            Self::Struct(o) => &o.ident,
            Self::Extern { ident, .. } => ident,
            Self::Unknown { ident, .. } => ident,
        }
    }

    pub fn generics(&self) -> &Generics {
        match self {
            Self::Enum(o) => &o.generics,
            Self::Struct(o) => &o.generics,
            Self::Extern { generics, .. } => generics,
            Self::Unknown { generics, .. } => generics,
        }
    }
}

impl AttribsEx for TypeToMock {
    fn derives(&self, ident: &str) -> bool {
        match self {
            Self::Enum(o) => o.derives(ident),
            Self::Struct(o) => o.derives(ident),
            _ => false,
        }
    }

    fn has_murf_attr(&self, ident: &str) -> bool {
        match self {
            Self::Enum(o) => o.has_murf_attr(ident),
            Self::Struct(o) => o.has_murf_attr(ident),
            _ => false,
        }
    }

    fn remove_murf_attrs(self) -> Self {
        match self {
            Self::Enum(o) => Self::Enum(o.remove_murf_attrs()),
            Self::Struct(o) => Self::Struct(o.remove_murf_attrs()),
            x => x,
        }
    }
}

impl Parse for TypeToMock {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let fork = input.fork();
        let ret = match fork.parse::<Item>()? {
            Item::Enum(o) => Ok(Self::Enum(o)),
            Item::Struct(o) => Ok(Self::Struct(o)),
            Item::Impl(i) => {
                if let Type::Path(p) = &*i.self_ty {
                    return Ok(Self::Unknown {
                        ident: p.path.segments.last().unwrap().ident.clone(),
                        generics: i.generics,
                    });
                } else {
                    Err(input.error("Unexpected type!"))
                }
            }
            _ => Err(input.error("Expected enum, struct or impl block!")),
        };

        if ret.is_ok() {
            input.advance_to(&fork);
        }

        ret
    }
}

impl ToTokens for TypeToMock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Enum(o) => o.to_tokens(tokens),
            Self::Struct(o) => o.to_tokens(tokens),
            _ => (),
        }
    }
}
