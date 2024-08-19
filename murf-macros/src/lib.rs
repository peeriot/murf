#![warn(
    clippy::pedantic,
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::no_effect_underscore_binding,
    clippy::similar_names
)]
#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "debug-to-file", feature(proc_macro_span))]

use expect_call::CallMode;
use proc_macro::TokenStream;

mod expect_call;
mod misc;
mod mock;

/// Macro to generate a mockable version of a type or trait.
///
/// # Example
///
/// The following example will generate a mocked version of `MyStruct` that
/// implements the `Fuu`  trait.
///
/// ```
/// trait Fuu {
///     fn fuu(&self) -> usize;
/// }
///
/// mock! {
///     #[derive(Default)]
///     pub struct MyStruct;
///
///     impl Fuu for MyStruct {
///         fn fuu(&self) -> usize;
///     }
/// }
///
/// let (handle, mock) = MyStruct::mock_with_handle();
///
/// expect_method_call!(handle as Fuu, fuu()).will_once(Return(1));
///
/// assert_eq!(1, mock.fuu());
/// ```
#[proc_macro]
#[cfg(not(doctest))]
pub fn mock(input: TokenStream) -> TokenStream {
    mock::exec(input.into()).into()
}

/// Helper macro to define an call expectation of a specific function.
///
/// # Example
///
/// ```
/// let (handle, mock) = MyStruct::mock_with_handle();
///
/// expect_call!(handle as Fuu, fuu(_)).will_once(Return(1));
/// ```
#[proc_macro]
#[cfg(not(doctest))]
pub fn expect_call(input: TokenStream) -> TokenStream {
    expect_call::exec(input.into(), CallMode::Static).into()
}

/// Helper macro to define an call expectation of a specific method. Same as
/// [`expect_call`] but will automatically add a `any` matcher for the `self`
/// argument.
///
/// # Example
///
/// ```
/// let (handle, mock) = MyStruct::mock_with_handle();
///
/// expect_method_call!(handle as Fuu, fuu()).will_once(Return(1));
/// ```
#[proc_macro]
#[cfg(not(doctest))]
pub fn expect_method_call(input: TokenStream) -> TokenStream {
    expect_call::exec(input.into(), CallMode::Method).into()
}
