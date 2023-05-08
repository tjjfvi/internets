use syn::{parse::Parse, Path, Token};

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
