use lazy_static::lazy_static;
use quote::ToTokens;
use regex::{Captures, Regex};

pub trait FormattedString {
    fn to_formatted_string(&self) -> String;
}

impl<X> FormattedString for X
where
    X: ToTokens,
{
    fn to_formatted_string(&self) -> String {
        let code = self.to_token_stream().to_string();
        let code = PATH_FORMAT_1.replace_all(&code, |c: &Captures| c[1].to_string());
        let code = PATH_FORMAT_2.replace_all(&code, "&");

        code.into_owned()
    }
}

lazy_static! {
    static ref PATH_FORMAT_1: Regex = Regex::new(r"\s*(<|>)\s*").unwrap();
    static ref PATH_FORMAT_2: Regex = Regex::new(r"&\s*").unwrap();
}
