use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated,
    token::{Colon, Comma, Gt, Lt},
    GenericArgument, GenericParam, Generics, Lifetime, LifetimeParam, PathArguments, Type,
    TypeParamBound, WherePredicate,
};

use crate::misc::IterEx;

use super::TypeEx;

pub(crate) trait GenericsEx {
    fn get_lifetime_mut(&mut self, lt: &str) -> Option<&mut LifetimeParam>;

    fn add_lifetime(self, lt: &str) -> Self;
    fn add_lifetime_bounds(self, lt: &str) -> Self;

    fn remove_lifetimes(self, lts: &Punctuated<Lifetime, Comma>) -> Self;
    fn remove_other(self, other: &Generics) -> Self;

    fn make_phantom_data(&self) -> TokenStream;

    fn merge(self, other: &Generics) -> Self;
    fn replace_self_type(self, type_: &Type, changed: &mut bool) -> Self;
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
            .filter_map(|param| match param {
                GenericParam::Lifetime(lt) => {
                    let lt = &lt.lifetime;

                    Some(quote!(& #lt ()))
                }
                GenericParam::Type(ty) => {
                    let ident = &ty.ident;

                    Some(quote!(#ident))
                }
                GenericParam::Const(_) => None,
            })
            .parenthesis();

        quote!(PhantomData<#params>)
    }

    fn merge(mut self, other: &Generics) -> Self {
        for p1 in &other.params {
            let mut merged = false;

            for p2 in &mut self.params {
                match (p1, p2) {
                    (GenericParam::Type(p1), GenericParam::Type(p2)) if p1.ident == p2.ident => {
                        p2.bounds.extend(p1.bounds.clone());

                        merged = true;
                        break;
                    }
                    (GenericParam::Lifetime(p1), GenericParam::Lifetime(p2))
                        if p1.lifetime.ident == p2.lifetime.ident =>
                    {
                        p2.bounds.extend(p1.bounds.clone());

                        merged = true;
                        break;
                    }
                    (_, _) => (),
                }
            }

            if !merged {
                self.params.push(p1.clone());
            }
        }

        self
    }

    fn replace_self_type(mut self, type_: &Type, changed: &mut bool) -> Self {
        let Some(where_clause) = &mut self.where_clause else {
            return self;
        };

        for p in &mut where_clause.predicates {
            let WherePredicate::Type(t) = p else {
                continue;
            };

            t.bounded_ty = t.bounded_ty.clone().replace_self_type(type_, changed);

            for b in &mut t.bounds {
                let TypeParamBound::Trait(t) = b else {
                    continue;
                };

                for s in &mut t.path.segments {
                    let PathArguments::AngleBracketed(a) = &mut s.arguments else {
                        continue;
                    };

                    for a in &mut a.args {
                        let GenericArgument::Type(t) = a else {
                            continue;
                        };

                        *t = t.clone().replace_self_type(type_, changed);
                    }
                }
            }
        }

        self
    }
}
