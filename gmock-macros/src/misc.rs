use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use syn::Path;

pub fn format_expect_call(method: &Ident, as_trait: Option<&Path>) -> Ident {
    if let Some(t) = as_trait {
        format_ident!(
            "as_{}_expect_{}",
            t.to_token_stream()
                .to_string()
                .replace(|c: char| !c.is_alphanumeric(), "_")
                .to_case(Case::Snake),
            method
        )
    } else {
        format_ident!("expect_{}", method.to_string())
    }
}

pub fn format_expect_module(method: &Ident, as_trait: Option<&Path>) -> Ident {
    if let Some(t) = as_trait {
        format_ident!(
            "mock_trait_{}_method_{}",
            t.to_token_stream()
                .to_string()
                .replace(|c: char| !c.is_alphanumeric(), "_")
                .to_case(Case::Snake),
            method
        )
    } else {
        format_ident!("mock_method_{}", method.to_string())
    }
}

pub fn format_expectations_field(ident: &Ident) -> Ident {
    format_ident!("{}_expectations", ident)
}
