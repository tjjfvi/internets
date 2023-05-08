use crate::*;
use syn::{braced, parenthesized, parse::Parse, Expr, Token};

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
  pub parts: Vec<NetAgentPart>,
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
    let parts;
    parenthesized!(parts in input);
    let parts = parts.parse_terminated(NetAgentPart::parse, Token![,])?;
    Ok(NetAgent {
      src,
      name,
      parts: parts.into_iter().collect(),
    })
  }
}

#[derive(Debug)]
pub enum NetAgentPart {
  Port(Ident),
  Payload(PayloadExpr),
}

impl NetAgentPart {
  pub fn port(&self) -> Option<&Ident> {
    match self {
      NetAgentPart::Port(x) => Some(x),
      _ => None,
    }
  }
}

impl Parse for NetAgentPart {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![$]) {
      input.parse().map(NetAgentPart::Payload)
    } else if lookahead.peek(Ident) {
      input.parse().map(NetAgentPart::Port)
    } else {
      Err(lookahead.error())
    }
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
