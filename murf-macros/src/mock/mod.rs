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

use mocked::Mocked;
use parsed::Parsed;

pub(crate) fn exec(input: TokenStream) -> TokenStream {
    let mock = match parse2::<Parsed>(input) {
        Ok(parsed) => parsed,
        Err(err) => return err.to_compile_error(),
    };

    #[cfg(feature = "debug-to-file")]
    let ident = mock.ty.ident().to_string();

    #[allow(clippy::let_and_return)]
    let tokens = Mocked::new(mock).into_token_stream();

    #[cfg(feature = "debug")]
    println!("\nmock!:\n{tokens:#}\n");

    #[cfg(feature = "debug-to-file")]
    let _ = debug_to_file(&tokens, &ident);

    tokens
}

#[cfg(feature = "debug-to-file")]
fn debug_to_file(tokens: &TokenStream, ident: &str) -> std::io::Result<()> {
    use std::fs::{create_dir_all, write};
    use std::path::PathBuf;

    use convert_case::{Case, Casing};
    use proc_macro::Span;

    let path = Span::call_site()
        .source_file()
        .path()
        .join(ident.to_case(Case::Snake));
    let path = PathBuf::from("./target/generated").join(path);

    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    write(path, tokens.to_string())?;

    Ok(())
}
