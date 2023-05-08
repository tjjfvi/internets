mod check;
mod compile;
mod parser;
pub(self) use parser::*;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::{format_ident, quote, quote_spanned};
use std::collections::{BTreeMap, BTreeSet};
use syn::{spanned::Spanned, Ident};

#[proc_macro_error]
#[proc_macro]
pub fn interactions(input: TokenStream1) -> TokenStream1 {
  let input = match syn::parse::<Program>(input) {
    Ok(data) => data,
    Err(err) => {
      return TokenStream1::from(err.to_compile_error());
    }
  };
  input.check();
  input.compile().into()
}
