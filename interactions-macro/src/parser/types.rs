use syn::{parse::Parse, Ident, Token, Type};

#[derive(Debug)]
pub struct PortType {
  pub sign: Sign,
  pub name: Ident,
}

impl Parse for PortType {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let sign: Sign = input.parse()?;
    let name: Ident = input.parse()?;
    Ok(PortType { sign, name })
  }
}

#[derive(Debug)]
pub struct PayloadType {
  pub dollar: Token![$],
  pub ty: Type,
}

impl Parse for PayloadType {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let dollar: Token![$] = input.parse()?;
    let ty: Type = input.parse()?;
    Ok(PayloadType { dollar, ty })
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
  Minus,
  Plus,
}

impl Parse for Sign {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![+]) {
      input.parse::<Token![+]>().map(|_| Sign::Plus)
    } else if lookahead.peek(Token![-]) {
      input.parse::<Token![-]>().map(|_| Sign::Minus)
    } else {
      Err(lookahead.error())
    }
  }
}
