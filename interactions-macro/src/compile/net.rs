use crate::*;

#[derive(Debug)]
pub struct NetCompilation<'a> {
  pub interactions_ty: TokenStream,
  pub interactions_var: TokenStream,
  pub agents: Vec<TokenStream>,
  pub links: Vec<TokenStream>,
  pub seen: BTreeSet<&'a Ident>,
  pub implicit_id: usize,
}

impl Program {
  pub fn new_net_compilation(
    &self,
    interactions_ty: TokenStream,
    interactions_var: TokenStream,
  ) -> NetCompilation {
    NetCompilation {
      interactions_ty,
      interactions_var,
      agents: Vec::new(),
      links: Vec::new(),
      seen: BTreeSet::new(),
      implicit_id: 0,
    }
  }

  pub fn finish_net_compilation(&self, comp: NetCompilation) -> TokenStream {
    let agents = comp.agents;
    let links = comp.links;
    quote!(
      #(#agents)*
      #(#links)*
    )
  }

  pub fn compile_net<'a>(&self, net: &'a Net, comp: &mut NetCompilation<'a>) {
    for agent in &net.agents {
      self.compile_net_agent(agent, comp, None);
    }
  }

  pub fn compile_net_agent<'a>(
    &self,
    agent: &'a NetAgent,
    comp: &mut NetCompilation<'a>,
    mut implicit: Option<Ident>,
  ) {
    let crate_path = self.crate_path();
    let src = self.quote_src(&agent.src);
    let name = &agent.name;
    let mut vars = vec![];
    let fields = self.compile_fields(
      &agent.fields,
      quote!(),
      agent.fields.values().map(|x| match x {
        NetAgentField::Implicit(token) => {
          if let Some(implicit) = implicit.take() {
            vars.push(implicit.clone());
            quote!(&mut #implicit)
          } else {
            emit_error!(token.span(), "unexpected implicit port");
            quote!(&mut ())
          }
        }
        NetAgentField::Port(x) => {
          let (e0, e1) = self.edge_idents(x);
          let e = if comp.seen.insert(x) {
            e0
          } else {
            comp.links.push(self.compile_link(&e0, &e1));
            e1
          };
          vars.push(e.clone());
          quote!(&mut #e)
        }
        NetAgentField::Payload(PayloadExpr { expr, .. }) => {
          quote!(#expr)
        }
        NetAgentField::Agent(agent) => {
          let (e0, e1) = self.implicit_idents(&mut comp.implicit_id);
          comp.links.push(self.compile_link(&e0, &e1));
          self.compile_net_agent(agent, comp, Some(e0));
          vars.push(e1.clone());
          quote!(&mut #e1)
        }
      }),
    );
    if implicit.is_some() {
      emit_error!(agent.name.span(), "missing implicit port")
    }
    let interactions_ty = &comp.interactions_ty;
    let interactions_var = &comp.interactions_var;
    comp.agents.push(quote!(
      #(let mut #vars = #crate_path::LinkHalf::Null;)*
      #crate_path::Construct::<#interactions_ty>::construct(
        #src #name #fields,
        net,
        #interactions_var,
      );
    ));
  }

  pub fn compile_link(&self, a: &Ident, b: &Ident) -> TokenStream {
    let crate_path = self.crate_path();
    quote!(
      #crate_path::Net::link(net, #a, #b);
    )
  }

  pub fn edge_idents(&self, ident: &Ident) -> (Ident, Ident) {
    (format_ident!("{}_0", ident), format_ident!("{}_1", ident))
  }

  pub fn implicit_idents(&self, implicit_id: &mut usize) -> (Ident, Ident) {
    let id = std::mem::replace(implicit_id, *implicit_id + 1);
    (format_ident!("__{}_0", id), format_ident!("__{}_1", id))
  }
}
