mod formatted_string;
mod generics_ex;
mod inputs_ex;
mod item_impl_ex;
mod method_ex;
mod return_type_ex;
mod temp_lifetimes;
mod type_ex;

use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use syn::Path;

pub use formatted_string::FormattedString;
pub use generics_ex::GenericsEx;
pub use inputs_ex::InputsEx;
pub use item_impl_ex::ItemImplEx;
pub use method_ex::MethodEx;
pub use return_type_ex::ReturnTypeEx;
pub use temp_lifetimes::TempLifetimes;
pub use type_ex::TypeEx;

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
