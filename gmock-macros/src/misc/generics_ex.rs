use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated,
    token::{Colon, Comma, Gt, Lt},
    GenericParam, Generics, Lifetime, LifetimeParam, TypeParamBound,
};

use crate::misc::IterEx;

pub trait GenericsEx {
    fn get_lifetime_mut(&mut self, lt: &str) -> Option<&mut LifetimeParam>;

    fn add_lifetime(self, lt: &str) -> Self;
    fn add_lifetime_bounds(self, lt: &str) -> Self;

    fn remove_lifetimes(self, lts: &Punctuated<Lifetime, Comma>) -> Self;
    fn remove_other(self, other: &Generics) -> Self;

    fn make_phantom_data(&self) -> TokenStream;
}

impl GenericsEx for Generics {
    fn get_lifetime_mut(&mut self, lt: &str) -> Option<&mut LifetimeParam> {
        for x in &mut self.params {
            if let GenericParam::Lifetime(x) = x {
                if x.lifetime.to_string() == lt {
                    return Some(x);
                }
            }
        }

        None
    }

    fn add_lifetime(mut self, lt: &str) -> Self {
        for x in &self.params {
            if matches!(x, GenericParam::Lifetime(x) if x.lifetime.to_string() == lt) {
                return self;
            }
        }

        if self.lt_token.is_none() {
            self.lt_token = Some(Lt::default());
        }

        self.params.insert(
            0,
            GenericParam::Lifetime(LifetimeParam::new(Lifetime::new(lt, Span::call_site()))),
        );

        if self.gt_token.is_none() {
            self.gt_token = Some(Gt::default());
        }

        self
    }

    fn add_lifetime_bounds(mut self, lt: &str) -> Self {
        self.params.iter_mut().for_each(|param| match param {
            GenericParam::Type(t) => {
                if t.colon_token.is_none() {
                    t.colon_token = Some(Colon::default());
                }

                t.bounds.push(TypeParamBound::Lifetime(Lifetime::new(
                    lt,
                    Span::call_site(),
                )));
            }
            GenericParam::Lifetime(t) if t.lifetime.ident != lt[1..] => {
                if t.colon_token.is_none() {
                    t.colon_token = Some(Colon::default());
                }

                t.bounds.push(Lifetime::new(lt, Span::call_site()));
            }
            _ => (),
        });

        self
    }

    fn remove_lifetimes(mut self, lts: &Punctuated<Lifetime, Comma>) -> Self {
        self.params = self
            .params
            .into_iter()
            .filter(|param| {
                if let GenericParam::Lifetime(p) = param {
                    for lt in lts {
                        if p.lifetime.ident == lt.ident {
                            return false;
                        }
                    }
                }

                true
            })
            .collect();

        self
    }

    fn remove_other(mut self, other: &Generics) -> Self {
        self.params = self
            .params
            .into_iter()
            .filter(|param| {
                for p in &other.params {
                    match (param, p) {
                        (GenericParam::Type(a), GenericParam::Type(b)) if a.ident == b.ident => {
                            return false
                        }
                        (GenericParam::Const(a), GenericParam::Const(b)) if a.ident == b.ident => {
                            return false
                        }
                        (GenericParam::Lifetime(a), GenericParam::Lifetime(b))
                            if a.lifetime == b.lifetime =>
                        {
                            return false
                        }
                        (_, _) => (),
                    }
                }

                true
            })
            .collect();

        self
    }

    fn make_phantom_data(&self) -> TokenStream {
        let params = self
            .params
            .iter()
            .map(|param| match param {
                GenericParam::Lifetime(lt) => {
                    let lt = &lt.lifetime;

                    quote!(& #lt ())
                }
                GenericParam::Type(ty) => {
                    let ident = &ty.ident;

                    quote!(#ident)
                }
                GenericParam::Const(ct) => {
                    let ident = &ct.ident;

                    quote!(#ident)
                }
            })
            .parenthesis();

        quote!(PhantomData<#params>)
    }
}
