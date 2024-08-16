use std::{cell::UnsafeCell, mem::transmute};

use proc_macro2::{Ident, Span};
use syn::{
    punctuated::Punctuated, GenericArgument, Lifetime, Path, PathArguments, PathSegment,
    ReturnType, Type, TypeParamBound, TypePath,
};

use super::TempLifetimes;

pub enum LifetimeReplaceMode<'x> {
    Mock,
    Temp(&'x mut TempLifetimes),
}

impl<'x> LifetimeReplaceMode<'x> {
    fn generate(&mut self) -> Lifetime {
        match self {
            Self::Mock => Lifetime::new("'mock", Span::call_site()),
            Self::Temp(tmp) => tmp.generate(),
        }
    }
}

pub trait TypeEx {
    fn from_ident(ident: Ident) -> Self;

    fn contains_lifetime(&self, lt: &Lifetime) -> bool;
    fn contains_self_type(&self) -> bool;

    fn replace_self_type(self, type_: &Type, changed: &mut bool) -> Self;
    fn replace_default_lifetime(self, mode: LifetimeReplaceMode<'_>) -> Self;

    fn make_static(self) -> Self;
}

impl TypeEx for Type {
    fn from_ident(ident: Ident) -> Self {
        let mut path = Path {
            leading_colon: None,
            segments: Punctuated::default(),
        };
        path.segments.push(PathSegment {
            ident,
            arguments: PathArguments::None,
        });

        Self::Path(TypePath { qself: None, path })
    }

    fn contains_lifetime(&self, lt: &Lifetime) -> bool {
        struct Visitor<'a> {
            lt: &'a Lifetime,
            result: bool,
        }

        impl<'a> TypeVisitor for Visitor<'a> {
            fn visit_lifetime(&mut self, lt: &UnsafeCell<Lifetime>) -> bool {
                let lt = unsafe { &*lt.get() };
                self.result = self.lt.ident == lt.ident || self.result;

                !self.result
            }
        }

        let mut visitor = Visitor { lt, result: false };

        visitor.visit(unsafe_cell_ref(self));

        visitor.result
    }

    fn contains_self_type(&self) -> bool {
        struct Visitor {
            result: bool,
        }

        impl TypeVisitor for Visitor {
            fn visit_type(&mut self, ty: &UnsafeCell<Type>) -> bool {
                let ty = unsafe { &*ty.get() };

                if let Type::Path(t) = ty {
                    if t.path.segments.len() == 1 && t.path.segments[0].ident == "Self" {
                        self.result = true;
                    }
                }

                !self.result
            }
        }

        let mut visitor = Visitor { result: false };

        visitor.visit(unsafe_cell_ref(self));

        visitor.result
    }

    fn replace_self_type(mut self, type_: &Type, changed: &mut bool) -> Self {
        struct Visitor<'a> {
            type_: &'a Type,
            changed: &'a mut bool,
        }

        impl<'a> TypeVisitor for Visitor<'a> {
            fn visit_type(&mut self, ty: &UnsafeCell<Type>) -> bool {
                let ty = unsafe { &mut *ty.get() };

                if let Type::Path(t) = ty {
                    if t.path.segments.len() == 1 && t.path.segments[0].ident == "Self" {
                        *ty = self.type_.clone();
                        *self.changed = true;
                    }
                }

                true
            }
        }

        let mut visitor = Visitor { type_, changed };

        visitor.visit(unsafe_cell_mut(&mut self));

        self
    }

    fn replace_default_lifetime(mut self, mode: LifetimeReplaceMode<'_>) -> Self {
        struct Visitor<'a> {
            mode: LifetimeReplaceMode<'a>,
        }

        impl<'a> TypeVisitor for Visitor<'a> {
            fn visit_type(&mut self, ty: &UnsafeCell<Type>) -> bool {
                let ty = unsafe { &mut *ty.get() };

                if let Type::Reference(r) = ty {
                    if r.lifetime.is_none() {
                        r.lifetime = Some(self.mode.generate());
                    }
                }

                true
            }

            fn visit_lifetime(&mut self, lt: &UnsafeCell<Lifetime>) -> bool {
                let lt = unsafe { &mut *lt.get() };

                if lt.ident == "_" {
                    *lt = self.mode.generate();
                }

                true
            }
        }

        let mut visitor = Visitor { mode };

        visitor.visit(unsafe_cell_mut(&mut self));

        self
    }

    fn make_static(mut self) -> Self {
        struct Visitor;

        impl TypeVisitor for Visitor {
            fn visit_type(&mut self, ty: &UnsafeCell<Type>) -> bool {
                let ty = unsafe { &mut *ty.get() };

                match ty {
                    Type::Path(ty) => {
                        for seg in &mut ty.path.segments {
                            match &mut seg.arguments {
                                PathArguments::None | PathArguments::Parenthesized(_) => (),
                                PathArguments::AngleBracketed(x) => {
                                    for arg in &mut x.args {
                                        if let GenericArgument::Lifetime(lt) = arg {
                                            lt.ident = Ident::new("static", Span::call_site());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Type::Reference(ty) => ty.lifetime = None,
                    _ => (),
                }

                true
            }
        }

        Visitor.visit(unsafe_cell_mut(&mut self));

        self
    }
}

trait TypeVisitor: Sized {
    fn visit_type(&mut self, ty: &UnsafeCell<Type>) -> bool {
        let _ty = ty;

        true
    }

    fn visit_lifetime(&mut self, lt: &UnsafeCell<Lifetime>) -> bool {
        let _lt = lt;

        true
    }

    fn visit(&mut self, ty: &UnsafeCell<Type>) -> bool {
        if !self.visit_type(ty) {
            return false;
        }

        let ty = unsafe { &*ty.get() };

        fn visit_path<X: TypeVisitor>(this: &mut X, path: &Path) -> bool {
            for seg in &path.segments {
                match &seg.arguments {
                    PathArguments::None => (),
                    PathArguments::AngleBracketed(x) => {
                        for arg in &x.args {
                            match arg {
                                GenericArgument::Type(t) => {
                                    if !this.visit(unsafe_cell_ref(t)) {
                                        return false;
                                    }
                                }
                                GenericArgument::Lifetime(lt) => {
                                    if !this.visit_lifetime(unsafe_cell_ref(lt)) {
                                        return false;
                                    }
                                }
                                GenericArgument::AssocType(t) => {
                                    if !this.visit(unsafe_cell_ref(&t.ty)) {
                                        return false;
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    PathArguments::Parenthesized(x) => {
                        for t in &x.inputs {
                            if !this.visit(unsafe_cell_ref(t)) {
                                return false;
                            }
                        }

                        match &x.output {
                            ReturnType::Type(_, t) => {
                                if !this.visit(unsafe_cell_ref(t)) {
                                    return false;
                                }
                            }
                            ReturnType::Default => (),
                        }
                    }
                }
            }

            true
        }

        match ty {
            Type::Path(ty) => visit_path(self, &ty.path),
            Type::Reference(t) => {
                if let Some(lt) = &t.lifetime {
                    if !self.visit_lifetime(unsafe_cell_ref(lt)) {
                        return false;
                    }
                }

                if !self.visit(unsafe_cell_ref(&t.elem)) {
                    return false;
                }

                true
            }
            Type::Array(t) => self.visit(unsafe_cell_ref(&t.elem)),
            Type::Slice(t) => self.visit(unsafe_cell_ref(&t.elem)),
            Type::Tuple(t) => {
                for t in &t.elems {
                    if !self.visit(unsafe_cell_ref(t)) {
                        return false;
                    }
                }

                true
            }
            Type::TraitObject(t) => {
                for b in &t.bounds {
                    match b {
                        TypeParamBound::Lifetime(lt) => {
                            if !self.visit_lifetime(unsafe_cell_ref(lt)) {
                                return false;
                            }
                        }
                        TypeParamBound::Trait(t) => {
                            if !visit_path(self, &t.path) {
                                return false;
                            }
                        }
                        _ => (),
                    }
                }

                true
            }
            _ => true,
        }
    }
}

fn unsafe_cell_ref<T>(value: &T) -> &UnsafeCell<T> {
    unsafe { transmute(value) }
}

fn unsafe_cell_mut<T>(value: &mut T) -> &UnsafeCell<T> {
    unsafe { transmute(value) }
}
