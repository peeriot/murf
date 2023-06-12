use quote::{quote, ToTokens};
use syn::{FnArg, ImplItemFn, Item, ReturnType, Stmt, Type};

use crate::misc::{FormattedString, TypeEx};

use super::context::{MethodContext, MethodContextData};

pub struct MockMethod(ImplItemFn);

impl MockMethod {
    pub fn render(context: &MethodContext, mut method: ImplItemFn) -> ImplItemFn {
        let MethodContextData {
            is_associated,
            impl_,
            trait_,
            ga_expectation,
            ident_expectation_module,
            ident_expectation_field,
            args,
            ..
        } = &**context;

        let locked = if *is_associated {
            quote! {
                let mut locked = #ident_expectation_module::EXPECTATIONS.lock();
            }
        } else {
            quote! {
                let mut locked = self.shared.lock();
            }
        };

        let expectations_iter = if *is_associated {
            quote!(&mut *locked)
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

        let self_ty = &impl_.self_ty;
        let (_ga_expectation_impl, ga_expectation_types, _ga_expectation_where) =
            ga_expectation.split_for_impl();

        let args_name = args.iter().map(|i| &i.pat);
        let args_name = quote! { ( #( #args_name ),* ) };

        let arg_types = args.iter().map(|i| &i.ty);
        let arg_types = quote! { ( #( #arg_types ),* ) };

        let default_args = method.sig.inputs.iter().map(|i| match i {
            FnArg::Receiver(r) if r.ty.to_formatted_string() == "Pin<&mut Self>" => {
                quote!(unsafe { std::pin::Pin::new_unchecked(&mut self.get_unchecked_mut().state) })
            }
            FnArg::Receiver(r) if r.ty.to_formatted_string() == "Arc<Self>" => {
                quote!(Arc::new(self.state.clone()))
            }
            FnArg::Receiver(r) if r.ty.to_formatted_string() == "&Arc<Self>" => {
                quote!(&Arc::new(self.state.clone()))
            }
            FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_some() => {
                quote!(&mut self.state)
            }
            FnArg::Receiver(r) if r.reference.is_some() => quote!(&self.state),
            FnArg::Receiver(_) => quote!(self.state),
            FnArg::Typed(t) => t.pat.to_token_stream(),
        });
        let default_args = quote!( #( #default_args ),* );

        let default_action = if let Some(t) = trait_ {
            let method = &method.sig.ident;

            quote!(<#self_ty as #t>::#method)
        } else {
            let t = &impl_.self_ty;
            let method = &method.sig.ident;

            quote!(#t::#method)
        };

        let result = match &method.sig.output {
            ReturnType::Default => quote!(ret),
            ReturnType::Type(_, t) => match &**t {
                Type::Reference(r)
                    if r.mutability.is_some() && r.elem.to_formatted_string() == "Self" =>
                {
                    quote!(&mut self)
                }
                Type::Reference(r) if r.elem.to_formatted_string() == "Self" => quote!(&self),
                t if t.to_formatted_string() == "Self" => quote!(Self {
                    state: ret,
                    shared: self.shared.clone()
                }),
                t if t.to_formatted_string() == "Box<Self>" => quote!(Box<Self {
                    state: ret,
                    shared: self.shared.clone()
                }>),
                t if t.contains_self_type() => quote!(Self::from_state(ret, self.shared.clone())),
                _ => quote!(ret),
            },
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
            let args = #args_name;

            let mut msg = String::new();
            let _ = writeln!(msg, #error);
            let _ = writeln!(msg, "Tried the following expectations:");

            for ex in #expectations_iter {
                #expectation_unwrap

                let _ = writeln!(msg, "- {}", ex);

                /* type matches? */
                if ex.args_type_id() != type_name::<#arg_types>() {
                    let _ = writeln!(msg, "    The type mismatched");
                    continue;
                }
                let _ = writeln!(msg, "    The type matched");

                let ex: &mut dyn gmock::Expectation = &mut **ex;
                let ex = unsafe { &mut *(ex as *mut dyn gmock::Expectation as *mut #ident_expectation_module::Expectation #ga_expectation_types) };

                /* value matches? */
                if !ex.matches(&args) {
                    let _ = writeln!(msg, "    but the value mismatched");
                    continue;
                }
                let _ = writeln!(msg, "    and the value matched");

                /* is done? */
                let all_sequences_done = !ex.sequences.is_empty() && ex.sequences.iter().all(|s| s.is_done());
                if ex.times.is_done() || all_sequences_done {
                    let _ = writeln!(msg, "    but it is already done");

                    continue;
                }
                let _ = writeln!(msg, "    and it is not done yet");

                /* is active? */
                let mut is_active = true;
                for seq_handle in &ex.sequences {
                    if !seq_handle.is_active() {
                        if take(&mut is_active) {
                            let _ = writeln!(msg, "    but it is not active yet");
                        }

                        let _ = writeln!(msg, "      sequence #{} has unsatisfied expectations", seq_handle.sequence_id());
                        for ex in seq_handle.unsatisfied() {
                            let _ = writeln!(msg, "        - {}", ex);
                        }
                    }
                }
                if !is_active {
                    continue;
                }

                /* execute */
                ex.times.increment();
                if ex.times.is_ready() {
                    for seq_handle in &ex.sequences {
                        seq_handle.set_ready();
                    }
                }

                let ret = if let Some(action) = &mut ex.action {
                    action.exec(args)
                } else {
                    drop(locked);

                    let #args_name = args;

                    #default_action(#default_args)
                };

                return #result;
            }

            println!("{}", msg);

            panic!(#error);
        }))];

        method
    }
}
