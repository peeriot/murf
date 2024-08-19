use proc_macro2::Span;
use quote::quote;
use syn::{Lifetime, ReturnType, Type};

use super::{type_ex::LifetimeReplaceMode, TypeEx};

pub(crate) trait ReturnTypeEx {
    fn to_action_return_type(&self, ty: &Type, need_mock_lt: &mut bool) -> Type;
}

impl ReturnTypeEx for ReturnType {
    fn to_action_return_type(&self, ty: &Type, need_mock_lt: &mut bool) -> Type {
        if let ReturnType::Type(_, t) = &self {
            let mut t = t
                .clone()
                .replace_self_type(ty, need_mock_lt)
                .replace_default_lifetime(LifetimeReplaceMode::Mock);
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
