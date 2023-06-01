use quote::quote;
use syn::{ImplItemMethod, Item, ReturnType, Stmt};

pub trait MethodEx {
    fn has_default_impl(&self) -> bool;
    fn need_default_impl(&self) -> bool;
}

impl MethodEx for ImplItemMethod {
    fn has_default_impl(&self) -> bool {
        let stmts = &self.block.stmts;

        let no_impl_block = self.block.stmts.len() == 1
            && matches!(self.block.stmts.last(), Some(Stmt::Item(Item::Verbatim(v))) if v.to_string() == ";");
        let generated_panic_impl_block = quote!( #( #stmts )* )
            .to_string()
            .contains("\"No default action specified!\"");

        !no_impl_block && !generated_panic_impl_block
    }

    fn need_default_impl(&self) -> bool {
        self.sig.output != ReturnType::Default
    }
}
