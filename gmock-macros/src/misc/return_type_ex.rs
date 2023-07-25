use proc_macro2::Span;
use quote::quote;
use syn::{Lifetime, ReturnType, Type};

use super::TypeEx;

pub trait ReturnTypeEx {
    fn to_action_return_type(&self, ty: &Type) -> Type;
    fn to_action_return_type_checked(&self, ty: &Type, need_mock_lt: &mut bool) -> Type;
}

impl ReturnTypeEx for ReturnType {
    fn to_action_return_type(&self, ty: &Type) -> Type {
        let mut changed = false;

        self.to_action_return_type_checked(ty, &mut changed)
    }

    fn to_action_return_type_checked(&self, ty: &Type, need_mock_lt: &mut bool) -> Type {
        if let ReturnType::Type(_, t) = &self {
            let mut t = t.clone().replace_self_type_checked(ty, need_mock_lt);
            if let Type::Reference(t) = &mut t {
                t.lifetime = Some(Lifetime::new("'mock", Span::call_site()));
                *need_mock_lt = true;
            }

            t
        } else {
            Type::Verbatim(quote!(()))
        }
    }
}
