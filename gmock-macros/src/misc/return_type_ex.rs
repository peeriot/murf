use proc_macro2::Span;
use quote::quote;
use syn::{Lifetime, ReturnType, Type};

use super::TypeEx;

pub trait ReturnTypeEx {
    fn to_action_return_type(&self, ty: &Type) -> Type;
}

impl ReturnTypeEx for ReturnType {
    fn to_action_return_type(&self, ty: &Type) -> Type {
        if let ReturnType::Type(_, t) = &self {
            let mut t = t.clone().replace_self_type(ty);
            if let Type::Reference(t) = &mut t {
                t.lifetime = Some(Lifetime::new("'mock", Span::call_site()));
            }

            t
        } else {
            Type::Verbatim(quote!(()))
        }
    }
}
