use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::{mock_module::MockModule, parsed::Parsed};

/// Mocked implementation of a mock! macro
pub(crate) struct Mocked {
    parsed: Parsed,
    mock_module: MockModule,
}

impl Mocked {
    pub(crate) fn new(parsed: Parsed) -> Self {
        let mock_module = MockModule::new(&parsed);

        Self {
            parsed,
            mock_module,
        }
    }
}

impl ToTokens for Mocked {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            parsed,
            mock_module,
        } = self;

        tokens.extend(quote! {
            #parsed
            #mock_module
        });
    }
}
