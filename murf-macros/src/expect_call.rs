use std::borrow::Cow;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    token::{Comma, Gt, Lt, PathSep},
    AngleBracketedGenericArguments, Expr, GenericArgument, Path, PathArguments,
    Result as ParseResult, Token, Type,
};

use crate::misc::{format_expect_call, IterEx};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum CallMode {
    Method,
    Static,
}

pub(crate) fn exec(input: TokenStream, mode: CallMode) -> TokenStream {
    let mut call: Call = match parse2(input) {
        Ok(mock) => mock,
        Err(err) => {
            return err.to_compile_error();
        }
    };
    call.mode = mode;

    call.into_token_stream()
}

struct Call {
    obj: Box<Expr>,
    as_trait: Option<Path>,
    method: Ident,
    generics: Punctuated<GenericArgument, Token![,]>,
    args: Punctuated<Expr, Comma>,
    mode: CallMode,
}

impl Parse for Call {
    fn parse(input: ParseStream<'_>) -> ParseResult<Self> {
        let obj = input.parse()?;

        let (obj, as_trait) = if let Expr::Cast(o) = obj {
            if let Type::Path(as_trait) = *o.ty {
                (o.expr, Some(as_trait.path))
            } else {
                return Err(input.error("Expect trait path"));
            }
        } else {
            (Box::new(obj), None)
        };

        input.parse::<Token![,]>()?;
        let method = input.parse::<Ident>()?;
        let generics = if input.peek(Token![::]) {
            AngleBracketedGenericArguments::parse_turbofish(input)?.args
        } else {
            Punctuated::default()
        };
        let content;
        parenthesized!(content in input);
        let args = content.parse_terminated(Expr::parse, Token![,])?;

        Ok(Self {
            obj,
            as_trait,
            method,
            generics,
            args,
            mode: CallMode::Static,
        })
    }
}

impl ToTokens for Call {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            obj,
            as_trait,
            method,
            generics,
            args,
            mode,
        } = self;

        let desc = quote!(format!("at {}:{}", file!(), line!()));
        let obj = obj.to_token_stream();
        let method = format_expect_call(method, as_trait.as_ref());
        let generics = as_trait
            .as_ref()
            .and_then(|t| t.segments.last())
            .and_then(|s| {
                if let PathArguments::AngleBracketed(a) = &s.arguments {
                    Some(a.args.clone())
                } else {
                    None
                }
            })
            .into_iter()
            .flatten()
            .chain(generics.iter().cloned())
            .collect::<Punctuated<_, Comma>>();
        let turbofish = if generics.is_empty() {
            None
        } else {
            Some(AngleBracketedGenericArguments {
                colon2_token: Some(PathSep::default()),
                lt_token: Lt::default(),
                args: generics,
                gt_token: Gt::default(),
            })
        };
        let args = if args.is_empty() && mode == &CallMode::Static {
            quote!(.with(murf::matcher::no_args()))
        } else {
            let call_method = mode == &CallMode::Method;
            let args = args.iter().map(|a| {
                if a.to_token_stream().to_string() == "_" {
                    Cow::Owned(Expr::Verbatim(quote!(murf::matcher::any())))
                } else {
                    Cow::Borrowed(a)
                }
            });
            let args = call_method
                .then(|| Cow::Owned(Expr::Verbatim(quote!(murf::matcher::any()))))
                .into_iter()
                .chain(args)
                .parenthesis();

            quote!(.with(murf::matcher::multi(#args)))
        };

        tokens.extend(quote! {
            #obj.mock_handle().#method #turbofish().description(#desc)#args
        });

        #[cfg(feature = "debug")]
        println!("\nexpect_call!:\n{tokens:#}\n");
    }
}
