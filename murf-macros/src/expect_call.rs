use std::borrow::Cow;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    token::Comma,
    Expr, ExprCall, Path, Result as ParseResult, Token, Type,
};

use crate::misc::format_expect_call;

pub fn exec(input: TokenStream) -> TokenStream {
    let call: Call = match parse2(input) {
        Ok(mock) => mock,
        Err(err) => {
            return err.to_compile_error();
        }
    };

    call.into_token_stream()
}

struct Call {
    obj: Box<Expr>,
    as_trait: Option<Path>,
    method: Ident,
    args: Punctuated<Expr, Comma>,
}

impl Parse for Call {
    fn parse(input: ParseStream) -> ParseResult<Self> {
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

        let call: ExprCall = input.parse()?;

        let method = if let Expr::Path(p) = *call.func {
            if let Some(method) = p.path.get_ident() {
                method.clone()
            } else {
                return Err(input.error("Expect method identifier"));
            }
        } else {
            return Err(input.error("Expect method identifier"));
        };

        let args = call.args;

        Ok(Self {
            obj,
            as_trait,
            method,
            args,
        })
    }
}

impl ToTokens for Call {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            obj,
            as_trait,
            method,
            args,
        } = self;

        let desc = quote!(format!("at {}:{}", file!(), line!()));
        let obj = obj.to_token_stream();
        let method = format_expect_call(method, as_trait.as_ref());
        let args = if args.is_empty() {
            quote!(.with(murf::matcher::no_args()))
        } else {
            let args = args.iter().map(|a| {
                if a.to_token_stream().to_string() == "_" {
                    Cow::Owned(Expr::Verbatim(quote!(murf::matcher::any())))
                } else {
                    Cow::Borrowed(a)
                }
            });

            quote!(.with(murf::matcher::multi((#( #args ),*))))
        };

        tokens.extend(quote! {
            #obj.#method().description(#desc)#args
        });

        #[cfg(feature = "debug")]
        println!("\nexpect_call!:\n{tokens:#}\n");
    }
}
