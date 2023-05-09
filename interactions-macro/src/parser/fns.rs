use crate::*;
use syn::{parenthesized, parse::Parse, Ident, Token, Visibility};

#[derive(Debug)]
pub struct Fn {
  pub vis: Visibility,
  pub name: Ident,
  pub parts: Vec<FnPart>,
  pub net: Net,
}

impl Parse for Fn {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let vis: Visibility = input.parse()?;
    let _: Token![fn] = input.parse()?;
    let name: Ident = input.parse()?;
    let inputs;
    parenthesized!(inputs in input);
    let inputs = inputs.parse_terminated(FnPart::parse, Token![,])?;
    let inputs = inputs.into_iter().collect::<Vec<_>>();
    let net: Net = input.parse()?;
    Ok(Fn {
      vis,
      name,
      parts: inputs,
      net,
    })
  }
}

#[derive(Debug)]
pub struct FnPart {
  pub name: Ident,
  pub ty: StructField,
}

impl Parse for FnPart {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let name: Ident = input.parse()?;
    let _: Token![:] = input.parse()?;
    let ty: StructField = input.parse()?;
    Ok(FnPart { name, ty })
  }
}

impl Fn {
  pub fn all_idents<'a>(&'a self) -> impl Iterator<Item = &'a Ident> {
    self.input_idents().chain(self.inner_idents())
  }
  pub fn input_idents<'a>(&'a self) -> impl Iterator<Item = &'a Ident> {
    self
      .parts
      .iter()
      .filter(|x| x.ty.port().is_some())
      .map(|x| &x.name)
  }
  pub fn inner_idents<'a>(&'a self) -> impl Iterator<Item = &'a Ident> {
    self
      .net
      .agents
      .iter()
      .flat_map(|x| x.fields.values())
      .filter_map(NetAgentField::port)
  }
}
