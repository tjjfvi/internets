use crate::*;
use itertools::Either;
use syn::{
  parse::Parse,
  token::{Brace, Paren},
  Expr, Ident, Pat, Token,
};

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
      .fields
      .values()
      .chain(self.right.fields.values())
      .flat_map(|f| match f {
        ImplAgentField::Port(x) => Some(Either::Left([x].into_iter())),
        ImplAgentField::Agent(a) => Some(Either::Right(a.all_idents())),
        _ => None,
      })
      .flatten()
      .chain(self.net.all_idents())
  }
}

#[derive(Debug)]
pub struct ImplAgent {
  pub src: Option<Ident>,
  pub name: Ident,
  pub fields: Fields<ImplAgentField>,
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
    let fields: Fields<ImplAgentField> = input.parse()?;
    Ok(ImplAgent { src, name, fields })
  }
}

#[derive(Debug)]
pub enum ImplAgentField {
  Implicit(Token![_]),
  Port(Ident),
  Payload(PayloadPat),
  Agent(NetAgent),
}

impl Parse for ImplAgentField {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![_]) {
      input.parse().map(ImplAgentField::Implicit)
    } else if lookahead.peek(Ident) {
      let fork = input.fork();
      let _: Ident = fork.parse()?;
      let lookahead = fork.lookahead1();
      if lookahead.peek(Paren) || lookahead.peek(Brace) || lookahead.peek(Token![::]) {
        input.parse().map(ImplAgentField::Agent)
      } else {
        input.parse().map(ImplAgentField::Port)
      }
    } else if lookahead.peek(Token![$]) {
      input.parse().map(ImplAgentField::Payload)
    } else {
      Err(lookahead.error())
    }
  }
}

impl TryFrom<Ident> for ImplAgentField {
  type Error = ();
  fn try_from(value: Ident) -> Result<Self, Self::Error> {
    Ok(ImplAgentField::Port(value))
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
