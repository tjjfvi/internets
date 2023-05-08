use crate::*;
use syn::{parenthesized, parse::Parse, Expr, Ident, Pat, Token};

#[derive(Debug)]
pub struct Impl {
  pub imp: Token![impl],
  pub left: ImplAgent,
  pub right: ImplAgent,
  pub cond: Option<Expr>,
  pub net: Net,
}

impl Parse for Impl {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let imp: Token![impl] = input.parse()?;
    let left: ImplAgent = input.parse()?;
    let _: Token![for] = input.parse()?;
    let right: ImplAgent = input.parse()?;
    let lookahead = input.lookahead1();
    let cond = if lookahead.peek(Token![if]) {
      let _: Token![if] = input.parse()?;
      let cond: Expr = input.parse()?;
      Some(cond)
    } else {
      None
    };
    let net: Net = input.parse()?;
    Ok(Impl {
      imp,
      left,
      right,
      cond,
      net,
    })
  }
}

impl Impl {
  pub fn all_idents<'a>(&'a self) -> impl Iterator<Item = &'a Ident> {
    self
      .left
      .parts
      .iter()
      .chain(self.right.parts.iter())
      .filter_map(ImplAgentPart::auxiliary)
      .chain(
        self
          .net
          .agents
          .iter()
          .flat_map(|x| x.parts.iter())
          .filter_map(NetAgentPart::port),
      )
  }
}

#[derive(Debug)]
pub struct ImplAgent {
  pub src: Option<Ident>,
  pub name: Ident,
  pub parts: Vec<ImplAgentPart>,
}

impl Parse for ImplAgent {
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
    let parts = parts.parse_terminated(ImplAgentPart::parse, Token![,])?;
    Ok(ImplAgent {
      src,
      name,
      parts: parts.into_iter().collect(),
    })
  }
}

#[derive(Debug)]
pub enum ImplAgentPart {
  Principal(Token![_]),
  Auxiliary(Ident),
  Payload(PayloadPat),
}

impl ImplAgentPart {
  pub fn auxiliary(&self) -> Option<&Ident> {
    match self {
      ImplAgentPart::Auxiliary(x) => Some(x),
      _ => None,
    }
  }
}

impl Parse for ImplAgentPart {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![_]) {
      input.parse().map(ImplAgentPart::Principal)
    } else if lookahead.peek(Ident) {
      input.parse().map(ImplAgentPart::Auxiliary)
    } else if lookahead.peek(Token![$]) {
      input.parse().map(ImplAgentPart::Payload)
    } else {
      Err(lookahead.error())
    }
  }
}

#[derive(Debug)]
pub struct PayloadPat {
  pub dollar: Token![$],
  pub pat: Pat,
}

impl Parse for PayloadPat {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let dollar: Token![$] = input.parse()?;
    let pat: Pat = Pat::parse_multi(input)?;
    Ok(PayloadPat { dollar, pat })
  }
}
