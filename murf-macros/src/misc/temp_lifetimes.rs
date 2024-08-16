use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};

use proc_macro2::Span;
use syn::{punctuated::Punctuated, token::Comma, Lifetime};

#[derive(Default, Debug, Clone)]
pub struct TempLifetimes(pub Punctuated<Lifetime, Comma>);

impl TempLifetimes {
    pub fn new(lifetimes: Punctuated<Lifetime, Comma>) -> Self {
        Self(lifetimes)
    }

    pub fn generate(&mut self) -> Lifetime {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);

        let lt = format!("'murf_tmp_{id}");
        let lt = Lifetime::new(&lt, Span::call_site());

        self.0.push(lt.clone());

        lt
    }
}

impl Deref for TempLifetimes {
    type Target = Punctuated<Lifetime, Comma>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

static NEXT: AtomicUsize = AtomicUsize::new(0);
