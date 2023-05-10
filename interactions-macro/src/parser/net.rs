use crate::*;
use itertools::Either;
use syn::{
  braced,
  parse::Parse,
  token::{Brace, Paren},
  Expr, Token,
};

#[derive(Debug)]
pub struct Net {
  pub agents: Vec<NetAgent>,
}

impl Net {
  pub fn all_idents(&self) -> impl Iterator<Item = &Ident> {
    self.agents.iter().flat_map(NetAgent::all_idents)
  }
}

impl NetAgent {
  pub fn all_idents(&self) -> impl Iterator<Item = &Ident> {
    self
      .fields
      .values()
      .flat_map(|f| match f {
        NetAgentField::Port(x) => Some(Either::Left([x].into_iter())),
        NetAgentField::Agent(x) => Some(Either::Right(
          x.all_idents().collect::<Vec<_>>().into_iter(),
        )),
        _ => None,
      })
      .flatten()
  }
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
  Implicit(Token!(_)),
  Port(Ident),
  Payload(PayloadExpr),
  Agent(NetAgent),
}

impl Parse for NetAgentField {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![_]) {
      input.parse().map(NetAgentField::Implicit)
    } else if lookahead.peek(Token![$]) {
      input.parse().map(NetAgentField::Payload)
    } else if lookahead.peek(Ident) {
      let fork = input.fork();
      let _: Ident = fork.parse()?;
      let lookahead = fork.lookahead1();
      if lookahead.peek(Paren) || lookahead.peek(Brace) {
        input.parse().map(NetAgentField::Agent)
      } else {
        input.parse().map(NetAgentField::Port)
      }
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
