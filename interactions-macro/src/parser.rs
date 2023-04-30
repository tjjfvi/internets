use syn::{braced, parenthesized, parse::Parse, Expr, Ident, Pat, Token, Type};

#[derive(Debug)]
pub struct Input {
  pub ty: Ident,
  pub items: Vec<Item>,
}

impl Parse for Input {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let _: Token![type] = input.parse()?;
    let ty: Ident = input.parse()?;
    let _: Token![;] = input.parse()?;
    let mut items: Vec<Item> = vec![];
    while !input.is_empty() {
      items.push(input.parse()?);
    }
    Ok(Input { ty, items })
  }
}

#[derive(Debug)]
pub enum Item {
  Struct(Struct),
  Impl(Impl),
  Fn(Fn),
}

impl Parse for Item {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![struct]) {
      input.parse().map(Item::Struct)
    } else if lookahead.peek(Token![impl]) {
      input.parse().map(Item::Impl)
    } else if lookahead.peek(Token![fn]) {
      input.parse().map(Item::Fn)
    } else {
      Err(lookahead.error())
    }
  }
}

#[derive(Debug)]
pub struct Struct {
  pub name: Ident,
  pub ports: Vec<PortType>,
  pub payload: Option<PayloadType>,
}

impl Parse for Struct {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let _: Token![struct] = input.parse()?;
    let name: Ident = input.parse()?;
    let types;
    parenthesized!(types in input);
    let types = types.parse_terminated(RawStructPart::parse, Token![,])?;
    let mut ports = vec![];
    let mut payload: Option<PayloadType> = None;
    for ty in types {
      if let Some(x) = payload {
        return Err(syn::Error::new(x.dollar.span, "payload type must be last"));
      }
      match ty {
        RawStructPart::Port(port) => {
          ports.push(port);
        }
        RawStructPart::Payload(payload_ty) => {
          payload = Some(payload_ty);
        }
      }
    }
    let _: Token![;] = input.parse()?;
    Ok(Struct {
      name,
      ports,
      payload,
    })
  }
}

#[derive(Debug)]
pub enum RawStructPart {
  Port(PortType),
  Payload(PayloadType),
}

impl Parse for RawStructPart {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![$]) {
      input.parse().map(RawStructPart::Payload)
    } else if input.peek(Token![+]) || input.peek(Token![-]) {
      input.parse().map(RawStructPart::Port)
    } else {
      Err(lookahead.error())
    }
  }
}

#[derive(Debug)]
pub struct PortType {
  pub sign: Sign,
  pub name: Ident,
}

impl Parse for PortType {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let sign: Sign = input.parse()?;
    let name: Ident = input.parse()?;
    Ok(PortType { sign, name })
  }
}

#[derive(Debug)]
pub struct PayloadType {
  pub dollar: Token![$],
  pub ty: Type,
}

impl Parse for PayloadType {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let dollar: Token![$] = input.parse()?;
    let ty: Type = input.parse()?;
    Ok(PayloadType { dollar, ty })
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
  Minus,
  Plus,
}

impl Parse for Sign {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![+]) {
      input.parse::<Token![+]>().map(|_| Sign::Plus)
    } else if lookahead.peek(Token![-]) {
      input.parse::<Token![-]>().map(|_| Sign::Minus)
    } else {
      Err(lookahead.error())
    }
  }
}

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
pub struct Impl {
  pub left: ImplAgent,
  pub right: ImplAgent,
  pub net: Net,
}

impl Parse for Impl {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let _: Token![impl] = input.parse()?;
    let left: ImplAgent = input.parse()?;
    let _: Token![for] = input.parse()?;
    let right: ImplAgent = input.parse()?;
    let net: Net = input.parse()?;
    Ok(Impl { left, right, net })
  }
}

#[derive(Debug)]
pub struct ImplAgent {
  pub name: Ident,
  pub aux: Vec<Ident>,
  pub payload: Option<PayloadPat>,
}

impl Parse for ImplAgent {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let name: Ident = input.parse()?;
    let parts;
    parenthesized!(parts in input);
    let parts = parts.parse_terminated(RawImplAgentPart::parse, Token![,])?;
    let mut underscore = false;
    let mut aux = vec![];
    let mut payload: Option<PayloadPat> = None;
    for part in parts {
      if let Some(payload) = payload {
        return Err(syn::Error::new(
          payload.dollar.span,
          "payload pat must be last",
        ));
      }
      match part {
        RawImplAgentPart::Underscore(token) => {
          if underscore {
            return Err(syn::Error::new(
              token.span,
              "underscore must only come at the beginning",
            ));
          }
          underscore = true;
        }
        RawImplAgentPart::Port(name) => {
          if !underscore {
            return Err(syn::Error::new(
              name.span(),
              "the principal port must be labeled `_`",
            ));
          }
          aux.push(name);
        }
        RawImplAgentPart::Payload(pat) => {
          if !underscore {
            return Err(syn::Error::new(
              pat.dollar.span,
              "the principal port must come first",
            ));
          }
          payload = Some(pat);
        }
      }
    }
    Ok(ImplAgent { name, aux, payload })
  }
}

#[derive(Debug)]
enum RawImplAgentPart {
  Underscore(Token![_]),
  Port(Ident),
  Payload(PayloadPat),
}

impl Parse for RawImplAgentPart {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![_]) {
      input.parse().map(RawImplAgentPart::Underscore)
    } else if lookahead.peek(Ident) {
      input.parse().map(RawImplAgentPart::Port)
    } else if lookahead.peek(Token![$]) {
      input.parse().map(RawImplAgentPart::Payload)
    } else {
      Err(lookahead.error())
    }
  }
}

#[derive(Debug)]
pub struct PayloadPat {
  pub dollar: Token![$],
  pub pat: Pat,
  pub cond: Option<Expr>,
}

impl Parse for PayloadPat {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let dollar: Token![$] = input.parse()?;
    let pat: Pat = Pat::parse_multi(input)?;
    let lookahead = input.lookahead1();
    let cond = if lookahead.peek(Token![if]) {
      let _: Token![if] = input.parse()?;
      let cond: Expr = input.parse()?;
      Some(cond)
    } else {
      None
    };
    Ok(PayloadPat { dollar, pat, cond })
  }
}

#[derive(Debug)]
pub struct NetAgent {
  pub name: Ident,
  pub ports: Vec<Ident>,
  pub payload: Option<PayloadExpr>,
}

impl Parse for NetAgent {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let name: Ident = input.parse()?;
    let parts;
    parenthesized!(parts in input);
    let parts = parts.parse_terminated(RawNetAgentPart::parse, Token![,])?;
    let mut ports = vec![];
    let mut payload: Option<PayloadExpr> = None;
    for pat in parts {
      if let Some(payload) = payload {
        return Err(syn::Error::new(payload.dollar.span, "payload must be last"));
      }
      match pat {
        RawNetAgentPart::Port(name) => {
          ports.push(name);
        }
        RawNetAgentPart::Payload(p) => {
          payload = Some(p);
        }
      }
    }
    Ok(NetAgent {
      name,
      ports,
      payload,
    })
  }
}

#[derive(Debug)]
enum RawNetAgentPart {
  Port(Ident),
  Payload(PayloadExpr),
}

impl Parse for RawNetAgentPart {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![$]) {
      input.parse().map(RawNetAgentPart::Payload)
    } else if lookahead.peek(Ident) {
      input.parse().map(RawNetAgentPart::Port)
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

#[derive(Debug)]
pub struct Fn {
  pub name: Ident,
  pub inputs: Vec<Ident>,
  pub net: Net,
}

impl Parse for Fn {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let _: Token![fn] = input.parse()?;
    let name: Ident = input.parse()?;
    let inputs;
    parenthesized!(inputs in input);
    let inputs = inputs.parse_terminated(Ident::parse, Token![,])?;
    let inputs = inputs.into_iter().collect::<Vec<_>>();
    let net: Net = input.parse()?;
    Ok(Fn { name, inputs, net })
  }
}
