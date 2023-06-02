use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream, Parser, Result as ParseResult},
    Attribute, Generics, ImplItem, Item, ItemEnum, ItemImpl, ItemStruct, Meta, NestedMeta, Stmt,
    Type,
};

use crate::misc::MethodEx;

/// Parsed code inside the mock! macro
pub struct Parsed {
    pub ty: TypeToMock,
    pub impls: Vec<ItemImpl>,

    pub derive_sync: bool,
    pub derive_send: bool,
}

impl Parsed {
    fn add_default_impl(impl_: &mut ItemImpl) {
        for i in &mut impl_.items {
            if let ImplItem::Method(m) = i {
                if !m.has_default_impl() {
                    if m.need_default_impl() {
                        m.block.stmts = vec![Stmt::Item(Item::Verbatim(quote!(
                            panic!("No default action specified!");
                        )))];
                    } else {
                        m.block.stmts.clear();
                    }

                    let attr =
                        Parser::parse2(Attribute::parse_outer, quote!(#[allow(unused_variables)]))
                            .unwrap();

                    m.attrs.extend(attr);
                }
            }
        }
    }

    fn remove_uneeded_derives(ty: &mut TypeToMock) {
        let attrs = match ty {
            TypeToMock::Enum(o) => Some(&mut o.attrs),
            TypeToMock::Struct(o) => Some(&mut o.attrs),
            _ => None,
        };

        if let Some(attrs) = attrs {
            for attr in attrs {
                if let Ok(Meta::List(mut ml)) = attr.parse_meta() {
                    let i = ml.path.get_ident();
                    if matches!(i, Some(i) if i == "derive") {
                        ml.nested = ml
                            .nested
                            .into_iter()
                            .filter(|nm| {
                                if let NestedMeta::Meta(m) = nm {
                                    match m.path().get_ident() {
                                        Some(i) if i == "Send" => false,
                                        Some(i) if i == "Sync" => false,
                                        _ => true,
                                    }
                                } else {
                                    true
                                }
                            })
                            .collect();
                        ml.path.leading_colon = None;
                        ml.path.segments.clear();

                        attr.tokens = ml.to_token_stream();
                    }
                }
            }
        }
    }
}

impl Parse for Parsed {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut ty = input.parse::<TypeToMock>()?;

        let derive_send = ty.derives("Send");
        let derive_sync = ty.derives("Sync");

        Self::remove_uneeded_derives(&mut ty);

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

            Self::add_default_impl(&mut impl_);

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
                i.to_tokens(tokens);
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

    pub fn attributes(&self) -> &[Attribute] {
        match self {
            Self::Enum(o) => &o.attrs,
            Self::Struct(o) => &o.attrs,
            Self::Extern { .. } => &[],
            Self::Unknown { .. } => &[],
        }
    }

    pub fn derives(&self, ident: &str) -> bool {
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
