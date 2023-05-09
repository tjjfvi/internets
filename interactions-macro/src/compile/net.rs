use crate::*;

impl Program {
  pub fn compile_net<'a>(
    &self,
    net: &'a Net,
    seen: &mut BTreeSet<&'a Ident>,
    interactions_ty: TokenStream,
    interactions_var: TokenStream,
  ) -> TokenStream {
    let agents = net
      .agents
      .iter()
      .map(move |x| self.compile_net_agent(&interactions_ty, &interactions_var, seen, x));
    quote!(#(#agents)*)
  }

  fn compile_net_agent<'a>(
    &self,
    interactions_ty: &TokenStream,
    interactions_var: &TokenStream,
    seen: &mut BTreeSet<&'a Ident>,
    agent: &'a NetAgent,
  ) -> TokenStream {
    let crate_path = self.crate_path();
    let src = self.quote_src(&agent.src);
    let name = &agent.name;
    let mut vars = vec![];
    let fields = self.compile_fields(
      &agent.fields,
      quote!(),
      agent.fields.values().map(|x| match x {
        NetAgentField::Port(x) => {
          let (e0, e1) = self.edge_idents(x);
          let e = if seen.insert(x) { e0 } else { e1 };
          vars.push(e.clone());
          quote!(&mut #e)
        }
        NetAgentField::Payload(PayloadExpr { expr, .. }) => {
          quote!(#expr)
        }
      }),
    );
    quote!(
      #(let mut #vars = #crate_path::LinkHalf::Null;)*
      #crate_path::Construct::<#interactions_ty>::construct(
        #src #name #fields,
        net,
        #interactions_var,
      );
    )
  }

  pub fn link_edge_idents<'a, I: Iterator<Item = &'a Ident>>(&self, idents: I) -> TokenStream {
    let links = idents
      .collect::<BTreeSet<_>>()
      .iter()
      .map(|ident| {
        let (id_0, id_1) = self.edge_idents(ident);
        quote!(net.link(#id_0, #id_1);)
      })
      .collect::<Vec<_>>();
    quote!(#(#links)*)
  }

  pub fn edge_idents(&self, ident: &Ident) -> (Ident, Ident) {
    (format_ident!("{}_0", ident), format_ident!("{}_1", ident))
  }
}
