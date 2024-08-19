use std::mem::take;

use syn::{punctuated::Punctuated, GenericParam, ItemImpl, WherePredicate};

use super::{TempLifetimes, TypeEx};

pub(crate) trait ItemImplEx: Sized {
    fn split_off_temp_lifetimes(self) -> (Self, TempLifetimes);
}

impl ItemImplEx for ItemImpl {
    fn split_off_temp_lifetimes(mut self) -> (Self, TempLifetimes) {
        let mut lts = Punctuated::default();

        let params = take(&mut self.generics.params);

        for param in params {
            match param {
                GenericParam::Lifetime(lt) if !self.self_ty.contains_lifetime(&lt.lifetime) => {
                    if let Some(wc) = &mut self.generics.where_clause {
                        wc.predicates = wc.predicates.iter().filter_map(|p| {
                            if matches!(p, WherePredicate::Lifetime(plt) if plt.lifetime == lt.lifetime) {
                                None
                            } else {
                                Some(p.clone())
                            }
                        }).collect();
                    }

                    lts.push(lt.lifetime);
                }
                param => self.generics.params.push(param),
            }
        }

        if self.generics.params.is_empty() {
            self.generics.lt_token = None;
            self.generics.gt_token = None;
        }

        if self
            .generics
            .where_clause
            .as_ref()
            .is_some_and(|wc| wc.predicates.is_empty())
        {
            self.generics.where_clause = None;
        }

        (self, TempLifetimes::new(lts))
    }
}
