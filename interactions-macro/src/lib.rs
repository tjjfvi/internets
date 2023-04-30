mod parser;
use parser::*;

use proc_macro::TokenStream as TokenStream1;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn interactions(input: TokenStream1) -> TokenStream1 {
  _interactions(input.into()).into()
}

fn _interactions(input: TokenStream1) -> TokenStream1 {
  let input = parse_macro_input!(input as Input);
  dbg!(input);
  quote!().into()
}
