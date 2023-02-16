use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use syn::Path;

pub fn format_expect_call(method: &Ident, as_trait: Option<&Path>) -> Ident {
    if let Some(t) = as_trait {
        format_ident!("as_{}_expect_{}", t.to_token_stream().to_string(), method)
    } else {
        format_ident!("expect_{}", method.to_string())
    }
}

pub fn format_expect_module(method: &Ident, as_trait: Option<&Path>) -> Ident {
    if let Some(t) = as_trait {
        format_ident!("{}_{}", t.to_token_stream().to_string(), method)
    } else {
        format_ident!("{}", method.to_string())
    }
}

pub fn format_expectations_field(ident: &Ident) -> Ident {
    format_ident!("{}_expectations", ident)
}
