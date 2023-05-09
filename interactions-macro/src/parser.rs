mod fields;
mod fns;
mod impls;
mod items;
mod net;
mod structs;
mod types;
mod uses;

pub use fields::*;
pub use fns::*;
pub use impls::*;
pub use items::*;
pub use net::*;
pub use structs::*;
pub use types::*;
pub use uses::*;

use syn::parse::Parse;

#[derive(Debug)]
pub struct Program {
  pub items: Vec<Item>,
}

impl Parse for Program {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut items: Vec<Item> = vec![];
    while !input.is_empty() {
      items.push(input.parse()?);
    }
    Ok(Program { items })
  }
}
