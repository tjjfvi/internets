use crate::*;
use syn::{parenthesized, parse::Parse, Ident, Token, Visibility};

#[derive(Debug)]
pub struct Struct {
  pub vis: Visibility,
  pub name: Ident,
  pub parts: Vec<StructPart>,
}

impl Struct {
  pub fn ports(&self) -> impl Iterator<Item = (usize, (usize, &PortType))> {
    self
      .parts
      .iter()
      .enumerate()
      .filter_map(|(i, x)| Some((i, x.port()?)))
      .enumerate()
  }
  pub fn payloads(&self) -> impl Iterator<Item = (usize, (usize, &PayloadType))> {
    self
      .parts
      .iter()
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
    let parts;
    parenthesized!(parts in input);
    let parts = parts.parse_terminated(StructPart::parse, Token![,])?;
    let _: Token![;] = input.parse()?;
    Ok(Struct {
      vis,
      name,
      parts: parts.into_iter().collect(),
    })
  }
}

#[derive(Debug)]
pub enum StructPart {
  Port(PortType),
  Payload(PayloadType),
}

impl StructPart {
  pub fn port(&self) -> Option<&PortType> {
    match self {
      StructPart::Port(x) => Some(x),
      _ => None,
    }
  }
  pub fn payload(&self) -> Option<&PayloadType> {
    match self {
      StructPart::Payload(x) => Some(x),
      _ => None,
    }
  }
}

impl Parse for StructPart {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![$]) {
      input.parse().map(StructPart::Payload)
    } else if input.peek(Token![+]) || input.peek(Token![-]) {
      input.parse().map(StructPart::Port)
    } else {
      Err(lookahead.error())
    }
  }
}
