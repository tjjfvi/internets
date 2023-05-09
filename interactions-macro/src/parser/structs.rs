use crate::*;
use syn::{parse::Parse, Ident, Token, Visibility};

#[derive(Debug)]
pub struct Struct {
  pub vis: Visibility,
  pub name: Ident,
  pub fields: Fields<StructField>,
}

impl Struct {
  pub fn ports(&self) -> impl Iterator<Item = (usize, (usize, &PortType))> {
    self
      .fields
      .values()
      .enumerate()
      .filter_map(|(i, x)| Some((i, x.port()?)))
      .enumerate()
  }
  pub fn payloads(&self) -> impl Iterator<Item = (usize, (usize, &PayloadType))> {
    self
      .fields
      .values()
      .enumerate()
      .filter_map(|(i, x)| Some((i, x.payload()?)))
      .enumerate()
  }
}

impl Parse for Struct {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let vis: Visibility = input.parse()?;
    let _: Token![struct] = input.parse()?;
    let name: Ident = input.parse()?;
    let fields: Fields<_> = input.parse()?;
    if fields.semi() {
      let _: Token![;] = input.parse()?;
    }
    Ok(Struct { vis, name, fields })
  }
}

#[derive(Debug)]
pub enum StructField {
  Port(PortType),
  Payload(PayloadType),
}

impl StructField {
  pub fn port(&self) -> Option<&PortType> {
    match self {
      StructField::Port(x) => Some(x),
      _ => None,
    }
  }
  pub fn payload(&self) -> Option<&PayloadType> {
    match self {
      StructField::Payload(x) => Some(x),
      _ => None,
    }
  }
}

impl Parse for StructField {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![$]) {
      input.parse().map(StructField::Payload)
    } else if input.peek(Token![+]) || input.peek(Token![-]) {
      input.parse().map(StructField::Port)
    } else {
      Err(lookahead.error())
    }
  }
}

impl TryFrom<Ident> for StructField {
  type Error = ();
  fn try_from(_: Ident) -> Result<Self, Self::Error> {
    Err(())
  }
}
