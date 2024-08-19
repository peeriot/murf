mod attribs_ex;
mod formatted_string;
mod generics_ex;
mod item_impl_ex;
mod iter_ex;
mod method_ex;
mod return_type_ex;
mod temp_lifetimes;
mod type_ex;

use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::format_ident;
use syn::Path;

pub(crate) use attribs_ex::AttribsEx;
pub(crate) use formatted_string::FormattedString;
pub(crate) use generics_ex::GenericsEx;
pub(crate) use item_impl_ex::ItemImplEx;
pub(crate) use iter_ex::IterEx;
pub(crate) use method_ex::MethodEx;
pub(crate) use return_type_ex::ReturnTypeEx;
pub(crate) use temp_lifetimes::TempLifetimes;
pub(crate) use type_ex::{LifetimeReplaceMode, TypeEx};

pub(crate) fn format_expect_call(method: &Ident, as_trait: Option<&Path>) -> Ident {
    if let Some(t) = as_trait {
        format_ident!(
            "as_{}_expect_{}",
            t.segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("_")
                .replace(|c: char| !c.is_alphanumeric(), "_")
                .to_case(Case::Snake),
            method
        )
    } else {
        format_ident!("expect_{}", method.to_string())
    }
}

pub(crate) fn format_expect_module(method: &Ident, as_trait: Option<&Path>) -> Ident {
    if let Some(t) = as_trait {
        format_ident!(
            "mock_trait_{}_method_{}",
            t.segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("_")
                .replace(|c: char| !c.is_alphanumeric(), "_")
                .to_case(Case::Snake),
            method
        )
    } else {
        format_ident!("mock_method_{}", method.to_string())
    }
}

pub(crate) fn format_expectations_field(ident: &Ident) -> Ident {
    format_ident!("{}_expectations", ident)
}

#[cfg(feature = "force-name")]
pub(crate) fn ident_murf() -> Ident {
    format_ident!("murf")
}

#[cfg(not(feature = "force-name"))]
pub(crate) fn ident_murf() -> Ident {
    use proc_macro_crate::{crate_name, FoundCrate};

    match crate_name("murf") {
        Ok(FoundCrate::Itself) => format_ident!("crate"),
        Ok(FoundCrate::Name(name)) => format_ident!("{name}"),
        Err(_) => format_ident!("murf"),
    }
}
