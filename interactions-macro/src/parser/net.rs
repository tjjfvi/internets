use crate::*;
use syn::{braced, parse::Parse, Expr, Token};

#[derive(Debug)]
pub struct Net {
  pub agents: Vec<NetAgent>,
}

impl Parse for Net {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let inner;
    braced!(inner in input);
    let mut agents: Vec<NetAgent> = vec![];
    while !inner.is_empty() {
      agents.push(inner.parse()?);
    }
    Ok(Net { agents })
  }
}

#[derive(Debug)]
pub struct NetAgent {
  pub src: Option<Ident>,
  pub name: Ident,
  pub fields: Fields<NetAgentField>,
}

impl Parse for NetAgent {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut src = None;
    let mut name: Ident = input.parse()?;
    if input.lookahead1().peek(Token![::]) {
      let _: Token![::] = input.parse()?;
      src = Some(name);
      name = input.parse()?;
    }
    let fields = input.parse()?;
    Ok(NetAgent { src, name, fields })
  }
}

#[derive(Debug)]
pub enum NetAgentField {
  Port(Ident),
  Payload(PayloadExpr),
}

impl NetAgentField {
  pub fn port(&self) -> Option<&Ident> {
    match self {
      NetAgentField::Port(x) => Some(x),
      _ => None,
    }
  }
}

impl Parse for NetAgentField {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![$]) {
      input.parse().map(NetAgentField::Payload)
    } else if lookahead.peek(Ident) {
      input.parse().map(NetAgentField::Port)
    } else {
      Err(lookahead.error())
    }
  }
}

impl TryFrom<Ident> for NetAgentField {
  type Error = ();
  fn try_from(value: Ident) -> Result<Self, Self::Error> {
    Ok(NetAgentField::Port(value))
  }
}

#[derive(Debug)]
pub struct PayloadExpr {
  pub dollar: Token![$],
  pub expr: Expr,
}

impl Parse for PayloadExpr {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let dollar: Token![$] = input.parse()?;
    let expr: Expr = input.parse()?;
    Ok(PayloadExpr { dollar, expr })
  }
}
