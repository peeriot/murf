#![cfg_attr(feature = "debug-to-file", feature(proc_macro_span))]

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
    expect_call::exec(input.into()).into()
}
