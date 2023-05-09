use crate::*;

impl Program {
  pub fn compile_fns(&self, includes: &TokenStream) -> TokenStream {
    let fns = self
      .items
      .iter()
      .filter_map(Item::as_fn)
      .map(|f| self.compile_fn(f, includes));
    quote!(#(#fns)*)
  }

  fn compile_fn(&self, f: &Fn, includes: &TokenStream) -> TokenStream {
    let crate_path = self.crate_path();
    let parts = f.parts.iter().map(|FnPart { ty: p, .. }| match p {
      StructField::Port(_) => quote!(pub &'a mut #crate_path::LinkHalf),
      StructField::Payload(PayloadType { ty, .. }) => quote!(pub #ty),
    });
    let binds = f.parts.iter().map(|FnPart { name, ty: p }| {
      let (_, e1) = self.edge_idents(name);
      match p {
        StructField::Port(_) => quote!(#e1),
        StructField::Payload(_) => quote!(#name),
      }
    });
    let sets = f
      .parts
      .iter()
      .filter_map(|x| Some((&x.name, x.ty.port()?)))
      .map(|(name, _)| {
        let (e0, e1) = self.edge_idents(name);
        quote!(*#e1 = #e0;)
      });
    let lifetime = f
      .parts
      .iter()
      .find(|x| x.ty.port().is_some())
      .map(|_| quote!('a,));
    let mut seen = BTreeSet::new();
    let agents = self.compile_net(&f.net, &mut seen, quote!(I), quote!(interactions));
    let inputs = f.input_idents().collect::<Vec<_>>();
    let links = self.link_edge_idents(f.inner_idents().filter(|x| !inputs.contains(x)));
    let name = &f.name;
    let vis = &f.vis;
    quote!(
      #[allow(non_camel_case_types)]
      #vis struct #name<#lifetime>(#(#parts),*);
      impl<#lifetime I: self::Use> #crate_path::Construct<I> for #name<#lifetime> {
        #[inline(always)]
        fn construct<N: #crate_path::Net>(self, net: &mut N, interactions: &I) {
          #includes
          let #name(#(#binds),*) = self;
          #agents
          #links
          #(#sets)*
        }
      }
    )
  }
}
