use syn::{
    parse::ParseStream, punctuated::Punctuated, token::Comma, Attribute, ImplItem, ImplItemFn,
    ItemEnum, ItemImpl, ItemStruct, Meta, Path,
};

/*AttribsEx */

pub(crate) trait AttribsEx: Sized {
    fn derives(&self, ident: &str) -> bool {
        let _ident = ident;

        false
    }

    fn has_murf_attr(&self, ident: &str) -> bool {
        let _ident = ident;

        false
    }

    fn remove_murf_attrs(self) -> Self {
        self
    }
}

impl AttribsEx for Vec<Attribute> {
    fn derives(&self, ident: &str) -> bool {
        self.iter().any(|attr| match &attr.meta {
            Meta::List(ml) if attr.path().is_ident("derive") => {
                let mut ret = false;

                let _ = ml.parse_args_with(|p: ParseStream<'_>| {
                    if let Ok(ml) = Punctuated::<Path, Comma>::parse_separated_nonempty(p) {
                        for p in &ml {
                            if p.is_ident(ident) {
                                ret = true;
                            }
                        }
                    }

                    Ok(())
                });

                ret
            }
            _ => false,
        })
    }

    fn has_murf_attr(&self, ident: &str) -> bool {
        self.iter().any(|attr| match &attr.meta {
            Meta::List(ml) if attr.path().is_ident("murf") => {
                let mut ret = false;

                let _ = ml.parse_args_with(|p: ParseStream<'_>| {
                    if let Ok(ml) = Punctuated::<Path, Comma>::parse_separated_nonempty(p) {
                        for p in &ml {
                            if p.is_ident(ident) {
                                ret = true;
                            }
                        }
                    }

                    Ok(())
                });

                ret
            }
            _ => false,
        })
    }

    fn remove_murf_attrs(mut self) -> Self {
        self.retain(|a| !a.path().is_ident("murf"));

        self
    }
}

impl AttribsEx for ItemEnum {
    fn derives(&self, ident: &str) -> bool {
        self.attrs.derives(ident)
    }

    fn has_murf_attr(&self, ident: &str) -> bool {
        self.attrs.has_murf_attr(ident)
    }

    fn remove_murf_attrs(mut self) -> Self {
        self.attrs = self.attrs.remove_murf_attrs();

        self
    }
}

impl AttribsEx for ItemStruct {
    fn derives(&self, ident: &str) -> bool {
        self.attrs.derives(ident)
    }

    fn has_murf_attr(&self, ident: &str) -> bool {
        self.attrs.has_murf_attr(ident)
    }

    fn remove_murf_attrs(mut self) -> Self {
        self.attrs = self.attrs.remove_murf_attrs();

        self
    }
}

impl AttribsEx for ItemImpl {
    fn remove_murf_attrs(mut self) -> Self {
        self.attrs = self.attrs.remove_murf_attrs();
        self.items = self
            .items
            .into_iter()
            .map(AttribsEx::remove_murf_attrs)
            .collect();

        self
    }
}

impl AttribsEx for ImplItem {
    fn remove_murf_attrs(self) -> Self {
        match self {
            Self::Fn(x) => Self::Fn(x.remove_murf_attrs()),
            x => x,
        }
    }
}

impl AttribsEx for ImplItemFn {
    fn has_murf_attr(&self, ident: &str) -> bool {
        self.attrs.has_murf_attr(ident)
    }

    fn remove_murf_attrs(mut self) -> Self {
        self.attrs = self.attrs.remove_murf_attrs();

        self
    }
}
