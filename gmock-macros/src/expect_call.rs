use std::borrow::Cow;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    token::{As, Comma},
    Expr, Path, Result as ParseResult, Token,
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
    obj: Ident,
    as_trait: Option<Path>,
    method: Ident,
    args: Punctuated<Expr, Comma>,
}

impl Parse for Call {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let obj = input.parse()?;
        let as_trait = if input.peek(As) {
            input.parse::<As>()?;

            Some(input.parse()?)
        } else {
            None
        };

        input.parse::<Token![,]>()?;
        let method = input.parse()?;

        let content;
        parenthesized!(content in input);

        let mut args = Punctuated::new();
        while !content.is_empty() {
            if !args.is_empty() {
                content.parse::<Token![,]>()?;
            }

            args.push(content.parse()?);
        }

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
            quote!(.with(gmock::matcher::unit()))
        } else {
            let args = args.iter().map(|a| {
                if a.to_token_stream().to_string() == "_" {
                    Cow::Owned(Expr::Verbatim(quote!(gmock::matcher::any())))
                } else {
                    Cow::Borrowed(a)
                }
            });

            quote!(.with(gmock::matcher::multi((#( #args ),*))))
        };

        tokens.extend(quote! {
            #obj.#method().description(#desc)#args
        });

        #[cfg(feature = "debug")]
        println!("\nexpect_call!:\n{tokens:#}\n");
    }
}
