use std::{cell::UnsafeCell, mem::transmute};

use proc_macro2::{Ident, Span};
use syn::{
    punctuated::Punctuated, GenericArgument, Lifetime, Path, PathArguments, PathSegment,
    ReturnType, Type, TypePath,
};

use super::TempLifetimes;

pub trait TypeEx {
    fn from_ident(ident: Ident) -> Self;

    fn contains_lifetime(&self, lt: &Lifetime) -> bool;
    fn contains_self_type(&self) -> bool;

    fn replace_self_type(self, type_: &Type) -> Self;
    fn replace_default_lifetime(self, lts: &mut TempLifetimes) -> Self;

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

    fn replace_self_type(mut self, type_: &Type) -> Self {
        struct Visitor<'a> {
            type_: &'a Type,
        }

        impl<'a> TypeVisitor for Visitor<'a> {
            fn visit_type(&mut self, ty: &UnsafeCell<Type>) -> bool {
                let ty = unsafe { &mut *ty.get() };

                if let Type::Path(t) = ty {
                    if t.path.segments.len() == 1 && t.path.segments[0].ident == "Self" {
                        *ty = self.type_.clone();
                    }
                }

                true
            }
        }

        let mut visitor = Visitor { type_ };

        visitor.visit(unsafe_cell_mut(&mut self));

        self
    }

    fn replace_default_lifetime(mut self, lts: &mut TempLifetimes) -> Self {
        struct Visitor<'a> {
            lts: &'a mut TempLifetimes,
        }

        impl<'a> TypeVisitor for Visitor<'a> {
            fn visit_lifetime(&mut self, lt: &UnsafeCell<Lifetime>) -> bool {
                let lt = unsafe { &mut *lt.get() };

                if lt.ident == "_" {
                    *lt = self.lts.generate();
                }

                true
            }
        }

        let mut visitor = Visitor { lts };

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

trait TypeVisitor {
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

        match ty {
            Type::Path(ty) => {
                for seg in &ty.path.segments {
                    match &seg.arguments {
                        PathArguments::None => (),
                        PathArguments::AngleBracketed(x) => {
                            for arg in &x.args {
                                match arg {
                                    GenericArgument::Type(t) => {
                                        if !self.visit(unsafe_cell_ref(t)) {
                                            return false;
                                        }
                                    }
                                    GenericArgument::Lifetime(lt) => {
                                        if !self.visit_lifetime(unsafe_cell_ref(lt)) {
                                            return false;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                        PathArguments::Parenthesized(x) => {
                            for t in &x.inputs {
                                if !self.visit(unsafe_cell_ref(t)) {
                                    return false;
                                }
                            }

                            match &x.output {
                                ReturnType::Type(_, t) => {
                                    if !self.visit(unsafe_cell_ref(t)) {
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
