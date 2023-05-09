use itertools::Either;
use proc_macro2::Span;
use syn::{
  braced, parenthesized,
  parse::Parse,
  punctuated::Punctuated,
  spanned::Spanned,
  token::{Brace, Comma, Paren},
  Ident, Token,
};

#[derive(Debug)]
pub enum Fields<T> {
  Unnamed(UnnamedFields<T>),
  Named(NamedFields<T>),
}

impl<T> Fields<T> {
  pub fn values(&self) -> impl Iterator<Item = &T> {
    match self {
      Fields::Unnamed(x) => Either::Left(x.values()),
      Fields::Named(x) => Either::Right(x.values()),
    }
  }
  pub fn len(&self) -> usize {
    match self {
      Fields::Unnamed(x) => x.len(),
      Fields::Named(x) => x.len(),
    }
  }
  pub fn semi(&self) -> bool {
    matches!(self, Fields::Unnamed(_))
  }
  pub fn span(&self) -> Span {
    match self {
      Fields::Unnamed(x) => x.paren.span.span(),
      Fields::Named(x) => x.brace.span.span(),
    }
  }
}

impl<T: Parse + TryFrom<Ident>> Parse for Fields<T> {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Paren) {
      input.parse().map(Fields::Unnamed)
    } else if lookahead.peek(Brace) {
      input.parse().map(Fields::Named)
    } else {
      Err(lookahead.error())
    }
  }
}

#[derive(Debug)]
pub struct UnnamedFields<T> {
  pub paren: Paren,
  pub entries: Punctuated<T, Comma>,
}

impl<T> UnnamedFields<T> {
  pub fn values(&self) -> impl Iterator<Item = &T> {
    self.entries.iter()
  }
  pub fn len(&self) -> usize {
    self.entries.len()
  }
}

impl<T: Parse> Parse for UnnamedFields<T> {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let inner;
    let paren = parenthesized!(inner in input);
    Ok(UnnamedFields {
      paren,
      entries: inner.parse_terminated(T::parse, Token![,])?,
    })
  }
}

#[derive(Debug)]
pub struct NamedFields<T> {
  pub brace: Brace,
  pub entries: Punctuated<NamedField<T>, Comma>,
}

impl<T> NamedFields<T> {
  pub fn values(&self) -> impl Iterator<Item = &T> {
    self.entries.iter().map(|x| &x.val)
  }
  pub fn len(&self) -> usize {
    self.entries.len()
  }
}

impl<T: Parse + TryFrom<Ident>> Parse for NamedFields<T> {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let inner;
    let brace = braced!(inner in input);
    Ok(NamedFields {
      brace,
      entries: inner.parse_terminated(Parse::parse, Token![,])?,
    })
  }
}

#[derive(Debug)]
pub struct NamedField<T> {
  pub key: Ident,
  pub val: T,
}

impl<T: Parse + TryFrom<Ident>> Parse for NamedField<T> {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let key: Ident = input.parse()?;
    let lookahead = input.lookahead1();
    let val = if lookahead.peek(Token![:]) {
      let _: Token![:] = input.parse()?;
      input.parse()?
    } else {
      key.clone().try_into().map_err(|_| lookahead.error())?
    };
    Ok(NamedField { key, val })
  }
}
