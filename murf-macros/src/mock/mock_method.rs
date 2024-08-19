use quote::{quote, ToTokens};
use syn::{FnArg, ImplItemFn, Item, PatType, ReturnType, Stmt, Type};

use crate::misc::{AttribsEx, FormattedString, IterEx, TypeEx};

use super::context::{MethodContext, MethodContextData};

pub(crate) struct MockMethod;

impl MockMethod {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn render(context: &MethodContext, mut method: ImplItemFn) -> ImplItemFn {
        let MethodContextData {
            is_associated,
            no_default_impl,
            impl_,
            trait_,
            ga_expectation,
            ident_expectation_module,
            ident_expectation_field,
            args,
            ret,
            args_prepared,
            type_signature,
            ..
        } = &**context;

        let locked = if *is_associated {
            quote! {
                let locked = #ident_expectation_module::EXPECTATIONS.lock();
            }
        } else {
            quote! {
                let shared = self.shared.clone();
                let mut locked = shared.lock();
            }
        };

        let expectations_iter = if *is_associated {
            quote! {
                murf::LocalContext::current()
                    .borrow()
                    .as_ref()
                    .into_iter()
                    .flat_map(|x| x.expectations(*#ident_expectation_module::TYPE_ID))
                    .chain(&*locked)
            }
        } else {
            quote!(&mut locked.#ident_expectation_field)
        };

        let expectation_unwrap = is_associated.then(|| {
            quote! {
                let ex = if let Some(ex) = ex.upgrade() {
                    ex
                } else {
                    continue;
                };
                let mut ex = ex.lock();
            }
        });

        let (_ga_expectation_impl, ga_expectation_types, _ga_expectation_where) =
            ga_expectation.split_for_impl();

        let arg_names = args
            .iter()
            .map(|arg| match arg {
                FnArg::Receiver(_) => quote!(self),
                FnArg::Typed(PatType { pat, .. }) => quote!( #pat ),
            })
            .parenthesis();

        let type_signature = type_signature.parenthesis();
        let arg_names_prepared = args_prepared.iter().map(|arg| &arg.pat).parenthesis();

        let default_args = method.sig.inputs.iter().map(|i| match i {
            FnArg::Receiver(r) if r.ty.to_formatted_string() == "Pin<&mut Self>" => {
                quote!(unsafe { std::pin::Pin::new_unchecked(&mut this.get_unchecked_mut().state) })
            }
            FnArg::Receiver(r) if r.ty.to_formatted_string() == "Arc<Self>" => {
                quote!(Arc::new(this.state.clone()))
            }
            FnArg::Receiver(r) if r.ty.to_formatted_string() == "&Arc<Self>" => {
                quote!(&Arc::new(this.state.clone()))
            }
            FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_some() => {
                quote!(&mut this.state)
            }
            FnArg::Receiver(r) if r.reference.is_some() => quote!(&this.state),
            FnArg::Receiver(_) => quote!(this.state),
            FnArg::Typed(t) => t.pat.to_token_stream(),
        });

        let default_action = if *no_default_impl {
            quote!(panic!("No default implementation for expectation {}", ex))
        } else if let Some(t) = trait_ {
            let ident = &method.sig.ident;
            let self_ty = &impl_.self_ty;
            let (_, ga_types, _) = method.sig.generics.split_for_impl();
            let turbofish = ga_types.as_turbofish();

            quote! {
                let #arg_names_prepared = args;
                let ret = <#self_ty as #t>::#ident #turbofish ( #( #default_args ),* );
            }
        } else {
            let t = &impl_.self_ty;
            let ident = &method.sig.ident;
            let (_, ga_types, _) = method.sig.generics.split_for_impl();
            let turbofish = ga_types.as_turbofish();

            quote! {
                let #arg_names_prepared = args;
                let ret = #t::#ident #turbofish ( #( #default_args ),* );
            }
        };

        let result = match ret {
            _ if *no_default_impl => None,
            ReturnType::Default => Some(quote!(ret)),
            ReturnType::Type(_, t) => Some(match &**t {
                Type::Reference(r)
                    if r.mutability.is_some() && r.elem.to_formatted_string() == "Self" =>
                {
                    quote!(&mut this)
                }
                Type::Reference(r) if r.elem.to_formatted_string() == "Self" => quote!(&this),
                t if t.to_formatted_string() == "Self" => quote!(Self {
                    state: ret,
                    shared: this.shared.clone(),
                    handle: this.handle.clone(),
                }),
                t if t.to_formatted_string() == "Box<Self>" => quote!(Box<Self {
                    state: ret,
                    shared: this.shared.clone(),
                    handle: this.handle.clone(),
                }>),
                t if t.contains_self_type() => {
                    let s = format!(
                        "No default implementation for `{}` for expectation {{}}",
                        t.to_formatted_string()
                    );

                    quote!(panic!(#s, ex))
                }
                _ => quote!(ret),
            }),
        };

        let error = if let Some(t) = trait_ {
            format!(
                "<{} as {}>",
                impl_.self_ty.to_token_stream(),
                t.to_formatted_string()
            )
        } else {
            format!("{}", impl_.self_ty.to_token_stream())
        };
        let error = format!(
            "No suitable expectation found for {}::{}",
            error, &method.sig.ident
        );

        method.block.stmts = vec![Stmt::Item(Item::Verbatim(quote! {
            #locked
            let args = #arg_names;

            let mut msg = String::new();
            let _ = writeln!(msg, #error);
            let _ = writeln!(msg, "Tried the following expectations:");

            for ex in #expectations_iter {
                #expectation_unwrap

                let _ = writeln!(msg, "- {}", ex);

                /* type matches? */
                if ex.type_signature() != type_name::<#type_signature>() {
                    let _ = writeln!(msg, "    Type signature:      not ok");
                    let _ = writeln!(msg, "        Expected:  `{}`", type_name::<#type_signature>());
                    let _ = writeln!(msg, "        But found: `{}`", ex.type_signature());

                    continue;
                }
                let _ = writeln!(msg, "    Type signature:      ok");

                let ex: &mut dyn murf::Expectation = &mut **ex;
                let ex = unsafe { &mut *(ex as *mut dyn murf::Expectation as *mut #ident_expectation_module::Expectation #ga_expectation_types) };

                let mut is_valid = true;

                /* value matches? */
                if !ex.matches(&args) {
                    let _ = writeln!(msg, "    Argument matcher:    not ok");

                    is_valid = false;
                } else {
                    let _ = writeln!(msg, "    Argument matcher:    ok");
                }

                /* is done? */
                if ex.times.is_done() {
                    let _ = writeln!(msg, "    Call count:          done");

                    is_valid = false;
                } else if ex.times.is_ready() {
                    let _ = writeln!(msg, "    Call count:          ready");
                } else {
                    let _ = writeln!(msg, "    Call count:          ok");
                }

                /* is active? */
                for seq_handle in &ex.sequences {
                    if seq_handle.is_done() {
                        is_valid = false;

                        let s = seq_handle.sequence_id().to_string();
                        let _ = writeln!(msg, "    Sequence #{}:{:>2$}done", s, "", 10 - s.len());
                    } else if seq_handle.is_active() {
                        let s = seq_handle.sequence_id().to_string();
                        let _ = writeln!(msg, "    Sequence #{}:{:>2$}active", s, "", 10 - s.len());
                    } else {
                        is_valid = false;

                        let s = seq_handle.sequence_id().to_string();
                        let _ = writeln!(msg, "    Sequence #{}:{:>2$}not active", s, "", 10 - s.len());
                        let _ = writeln!(msg, "        has unsatisfied expectations");

                        for ex in seq_handle.unsatisfied() {
                            let _ = writeln!(msg, "          - {}", ex);
                        }
                    }
                }

                if !is_valid {
                    continue;
                }

                /* execute */
                ex.times.increment();
                if ex.times.is_ready() {
                    for seq_handle in &ex.sequences {
                        seq_handle.set_ready();
                    }
                }

                return if let Some(action) = &mut ex.action {
                    action.exec(args)
                } else {
                    #default_action
                    #result
                };
            }

            println!("{}", msg);

            panic!(#error);
        }))];

        method.remove_murf_attrs()
    }
}
