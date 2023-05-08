use crate::*;

impl Program {
  pub fn compile_uses(&self) -> (TokenStream, TokenStream, Vec<TokenStream>) {
    let mut impls = vec![];
    let mut traits = vec![];
    let mut includes = vec![];
    for u in self.items.iter().filter_map(Item::as_use) {
      let path = &u.path;
      impls.push(quote!(
        impl #path::Use for Interactions {
          const KIND_START: u32 = 0 #(+ <Self as #traits>::KIND_COUNT)*;
        }
      ));
      let span = path.span();
      includes.push(quote_spanned!(span=>
        #[allow(unused)]
        use #path;
        #[allow(unused)]
        use #path::*;
      ));
      traits.push(quote!(#path::Use));
    }
    (quote!(#(#impls)*), quote!(#(#includes)*), traits)
  }

  pub fn quote_src(&self, src: &Option<Ident>) -> TokenStream {
    src
      .as_ref()
      .map(|trait_name| quote!(#trait_name::))
      .unwrap_or(quote!())
  }
}
