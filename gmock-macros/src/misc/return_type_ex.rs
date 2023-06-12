use proc_macro2::Span;
use quote::quote;
use syn::{parse2, Lifetime, ReturnType, Type};

use crate::mock::TypeToMock;

use super::TypeEx;

pub trait ReturnTypeEx {
    fn to_action_return_type(&self, ty: &TypeToMock) -> Type;
}

impl ReturnTypeEx for ReturnType {
    fn to_action_return_type(&self, ty: &TypeToMock) -> Type {
        if let ReturnType::Type(_, t) = &self {
            let ident = ty.ident();
            let (_ga_impl, ga_types, _ga_where) = ty.generics().split_for_impl();
            let ty = parse2(quote!(#ident #ga_types)).unwrap();

            let mut t = t.clone().replace_self_type(&ty);
            if let Type::Reference(t) = &mut t {
                t.lifetime = Some(Lifetime::new("'mock", Span::call_site()));
            }

            t
        } else {
            Type::Verbatim(quote!(()))
        }
    }
}
