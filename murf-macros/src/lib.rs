#![warn(
    unused,
    clippy::pedantic,
    future_incompatible,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    rust_2021_compatibility
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::no_effect_underscore_binding,
    clippy::similar_names
)]
#![cfg_attr(feature = "debug-to-file", feature(proc_macro_span))]

use expect_call::CallMode;
use proc_macro::TokenStream;

mod expect_call;
mod misc;
mod mock;

#[proc_macro]
pub fn mock(input: TokenStream) -> TokenStream {
    mock::exec(input.into()).into()
}

#[proc_macro]
pub fn expect_call(input: TokenStream) -> TokenStream {
    expect_call::exec(input.into(), CallMode::Static).into()
}

#[proc_macro]
pub fn expect_method_call(input: TokenStream) -> TokenStream {
    expect_call::exec(input.into(), CallMode::Method).into()
}
