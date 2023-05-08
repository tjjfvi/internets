use crate::*;
use syn::{parse::Parse, Token, Visibility};

#[derive(Debug)]
pub enum Item {
  Struct(Struct),
  Impl(Impl),
  Fn(Fn),
  Use(Use),
}

impl Item {
  pub fn as_struct(&self) -> Option<&Struct> {
    match self {
      Item::Struct(x) => Some(x),
      _ => None,
    }
  }
  pub fn as_impl(&self) -> Option<&Impl> {
    match self {
      Item::Impl(x) => Some(x),
      _ => None,
    }
  }
  pub fn as_fn(&self) -> Option<&Fn> {
    match self {
      Item::Fn(x) => Some(x),
      _ => None,
    }
  }
  pub fn as_use(&self) -> Option<&Use> {
    match self {
      Item::Use(x) => Some(x),
      _ => None,
    }
  }
}

impl Parse for Item {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let fork = input.fork();
    let _: Visibility = fork.parse()?;
    let lookahead = fork.lookahead1();
    if lookahead.peek(Token![struct]) {
      input.parse().map(Item::Struct)
    } else if lookahead.peek(Token![impl]) {
      input.parse().map(Item::Impl)
    } else if lookahead.peek(Token![fn]) {
      input.parse().map(Item::Fn)
    } else if lookahead.peek(Token![use]) {
      input.parse().map(Item::Use)
    } else {
      Err(lookahead.error())
    }
  }
}
