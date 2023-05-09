use crate::*;

impl Program {
  pub fn compile_fields<T>(
    &self,
    fields: &Fields<T>,
    prefix: TokenStream,
    values: impl Iterator<Item = TokenStream>,
  ) -> TokenStream {
    match fields {
      Fields::Unnamed(_) => quote_spanned!(fields.span()=> (#(#prefix #values),*)),
      Fields::Named(f) => {
        let entries = f
          .entries
          .iter()
          .map(|x| &x.key)
          .zip(values)
          .map(|(k, v)| quote!(#prefix #k: #v));
        quote_spanned!(fields.span()=> {#(#entries),*})
      }
    }
  }
}
