use std::ops::Not;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ItemImpl;

use crate::misc::{FormattedString, GenericsEx, IterEx, TempLifetimes};

use super::context::{ContextData, ImplContextData, MethodContext, MethodContextData};

pub struct Expectation {
    context: MethodContext,

    display: String,
    default_matcher: String,
}

impl Expectation {
    pub fn new(context: MethodContext, impl_: &ItemImpl) -> Self {
        let display = if let Some(trait_) = &context.trait_ {
            format!(
                "<{} as {}>::{}",
                context.impl_.self_ty.to_token_stream(),
                trait_.to_formatted_string(),
                context.ident_method,
            )
        } else {
            format!(
                "{}::{}",
                impl_.self_ty.to_token_stream(),
                context.ident_method
            )
        };

        let default_matcher = format!(
            "({})",
            context
                .args_prepared
                .iter()
                .map(|_| "_")
                .collect::<Vec<_>>()
                .join(", ")
        );

        Expectation {
            context,

            display,
            default_matcher,
        }
    }
}

impl ToTokens for Expectation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            context,

            display,
            default_matcher,
        } = self;

        let MethodContextData {
            context,
            is_associated,
            ga_expectation,
            lts_temp: TempLifetimes(lts_temp),
            lts_mock: TempLifetimes(lts_mock),

            args_prepared,
            args_prepared_lt,

            return_type,
            type_signature,
            ..
        } = &**context;

        let ImplContextData { context, .. } = &**context;

        let ContextData {
            trait_send,
            trait_sync,
            ..
        } = &**context;

        let trait_send = is_associated
            .then(|| quote!( + Send))
            .or_else(|| trait_send.clone());
        let trait_sync = is_associated
            .then(|| quote!( + Sync))
            .or_else(|| trait_sync.clone());

        let lts_temp = lts_temp.is_empty().not().then(|| quote!(< #lts_temp >));
        let lts_mock = lts_mock.is_empty().not().then(|| quote!(for < #lts_mock >));
        let lt = if *is_associated {
            quote!(+ 'static)
        } else {
            quote!(+ 'mock)
        };

        let type_signature = type_signature.parenthesis();
        let arg_types_prepared = args_prepared.iter().map(|t| &t.ty).parenthesis();
        let arg_types_prepared_lt = args_prepared_lt.iter().map(|t| &t.ty).parenthesis();

        let ga_expectation_phantom = ga_expectation.make_phantom_data();
        let (ga_expectation_impl, ga_expectation_types, ga_expectation_where) =
            ga_expectation.split_for_impl();

        tokens.extend(quote! {
            pub struct Expectation #ga_expectation_impl #ga_expectation_where {
                pub times: Times,
                pub description: Option<String>,
                pub action: Option<Box<dyn #lts_mock RepeatableAction<#arg_types_prepared_lt, #return_type> #trait_send #trait_sync #lt>>,
                pub matcher: Option<Box<dyn #lts_mock Matcher<#arg_types_prepared_lt> #trait_send #trait_sync #lt>>,
                pub sequences: Vec<SequenceHandle>,
                _marker: #ga_expectation_phantom,
            }

            impl #ga_expectation_impl Default for Expectation #ga_expectation_types #ga_expectation_where {
                fn default() -> Self {
                    Self {
                        times: Times::default(),
                        description: None,
                        action: None,
                        matcher: None,
                        sequences: Vec::new(),
                        _marker: PhantomData,
                    }
                }
            }

            impl #ga_expectation_impl Expectation #ga_expectation_types #ga_expectation_where {
                pub fn matches #lts_temp (&self, args: &#arg_types_prepared) -> bool {
                    if let Some(m) = &self.matcher {
                        m.matches(args)
                    } else {
                        true
                    }
                }
            }

            impl #ga_expectation_impl gmock::Expectation for Expectation #ga_expectation_types #ga_expectation_where {
                fn type_id(&self) -> usize {
                    *TYPE_ID
                }

                fn is_ready(&self) -> bool {
                    self.times.is_ready()
                }

                fn set_done(&self) {
                    for seq_handle in &self.sequences {
                        seq_handle.set_done();
                    }
                }

                fn type_signature(&self) -> &'static str {
                    type_name::<#type_signature>()
                }
            }

            impl #ga_expectation_impl Display for Expectation #ga_expectation_types #ga_expectation_where {
                fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                    write!(f, #display)?;

                    if let Some(m) = &self.matcher {
                        write!(f, "(")?;
                        m.fmt(f)?;
                        write!(f, ")")?;
                    } else {
                        write!(f, #default_matcher)?;
                    }

                    if let Some(d) = &self.description {
                        write!(f, " {}", d)?;
                    }

                    Ok(())
                }
            }
        })
    }
}
