use syn::{braced, parenthesized, parse::Parse, Expr, Ident, Pat, Path, Token, Type, Visibility};

#[derive(Debug)]
pub struct Input {
  pub items: Vec<Item>,
}

impl Parse for Input {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut items: Vec<Item> = vec![];
    while !input.is_empty() {
      items.push(input.parse()?);
    }
    Ok(Input { items })
  }
}

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

#[derive(Debug)]
pub struct Struct {
  pub vis: Visibility,
  pub name: Ident,
  pub parts: Vec<StructPart>,
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
  pub ty: StructPart,
}

impl Parse for FnPart {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let name: Ident = input.parse()?;
    let _: Token![:] = input.parse()?;
    let ty: StructPart = input.parse()?;
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
      .flat_map(|x| x.parts.iter())
      .filter_map(NetAgentPart::port)
  }
}

#[derive(Debug)]
pub struct Use {
  pub path: Path,
}

impl Parse for Use {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let _: Token![use] = input.parse()?;
    let path: Path = input.parse()?;
    let _: Token![;] = input.parse()?;
    Ok(Use { path })
  }
}
