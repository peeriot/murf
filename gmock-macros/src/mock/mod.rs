#![allow(clippy::module_inception)]
mod context;
mod expectation;
mod expectation_builder;
mod expectation_module;
mod handle;
mod mock;
mod mock_method;
mod mock_module;
mod mockable;
mod mockable_default;
mod mocked;
mod parsed;
mod shared;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse2;

pub use parsed::TypeToMock;

use mocked::Mocked;
use parsed::Parsed;

pub fn exec(input: TokenStream) -> TokenStream {
    let mock = match parse2::<Parsed>(input) {
        Ok(parsed) => parsed,
        Err(err) => return err.to_compile_error(),
    };

    #[allow(clippy::let_and_return)]
    let tokens = Mocked::new(mock).into_token_stream();

    #[cfg(feature = "debug")]
    println!("\nmock!:\n{tokens:#}\n");

    #[cfg(feature = "debug-to-file")]
    let _ = debug_to_file(&tokens);

    tokens
}

#[cfg(feature = "debug-to-file")]
fn debug_to_file(tokens: &TokenStream) -> std::io::Result<()> {
    use std::fs::{create_dir_all, write};
    use std::path::PathBuf;

    use proc_macro::Span;

    let path = Span::call_site().source_file().path();
    let path = PathBuf::from("./target/generated").join(path);

    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    write(path, tokens.to_string())?;

    Ok(())
}
