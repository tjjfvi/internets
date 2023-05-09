use crate::*;

impl Program {
  pub fn compile_impls(&self) -> Vec<TokenStream> {
    collect_multi_map(self.items.iter().filter_map(Item::as_impl).flat_map(|i| {
      let a = &i.left;
      let b = &i.right;
      [
        ((&a.src, &a.name, &b.src, &b.name), (a, b, i)),
        ((&b.src, &b.name, &a.src, &a.name), (b, a, i)),
      ]
    }))
    .into_iter()
    .map(|(k, v)| self.compile_impl_group(k, v))
    .collect::<Vec<_>>()
  }

  fn compile_impl_group(
    &self,
    (a_src, a_name, b_src, b_name): (&Option<Ident>, &Ident, &Option<Ident>, &Ident),
    impls: Vec<(&ImplAgent, &ImplAgent, &Impl)>,
  ) -> TokenStream {
    let crate_path = self.crate_path();
    let a_src = self.quote_src(&a_src);
    let b_src = self.quote_src(&b_src);
    let arms = impls
      .into_iter()
      .map(|(a, b, i)| self.compile_impl(a, b, i));
    let a_kind_path = quote!(<#a_src #a_name<_> as #crate_path::GetKind<Self>>::KIND);
    let b_kind_path = quote!(<#b_src #b_name<_> as #crate_path::GetKind<Self>>::KIND);
    quote!(
      x if (#a_kind_path <= #b_kind_path) && x == (#a_kind_path, #b_kind_path) => {
        match (
          <#a_src #a_name<_> as #crate_path::Destruct>::destruct(net, a_addr),
          <#b_src #b_name<_> as #crate_path::Destruct>::destruct(net, b_addr),
        ) {
          #(#arms)*
        }
        <#a_src #a_name<_> as #crate_path::Destruct>::free(net, a_addr);
        <#b_src #b_name<_> as #crate_path::Destruct>::free(net, b_addr);
      }
    )
  }

  fn compile_impl(&self, a: &ImplAgent, b: &ImplAgent, i: &Impl) -> TokenStream {
    let a_src = self.quote_src(&a.src);
    let b_src = self.quote_src(&b.src);
    let a_name = &a.name;
    let b_name = &b.name;
    let cond = i.cond.as_ref().map(|x| quote!(if #x)).unwrap_or(quote!());

    let mut seen = BTreeSet::new();
    let a_fields = self.impl_agent_fields(a, &mut seen);
    let b_fields = self.impl_agent_fields(b, &mut seen);
    let agents = self.compile_net(&i.net, &mut seen, quote!(Self), quote!(self));
    let links = self.link_edge_idents(i.all_idents());

    let a_pat = quote_spanned!(a.name.span()=> #a_src #a_name #a_fields);
    let b_pat = quote_spanned!(b.name.span()=> #b_src #b_name #b_fields);

    quote_spanned!(i.imp.span=>
      (#a_pat, #b_pat) #cond => {
        #agents
        #links
      }
    )
  }

  fn impl_agent_fields<'a>(&self, a: &'a ImplAgent, seen: &mut BTreeSet<&'a Ident>) -> TokenStream {
    self.compile_fields(
      &a.fields,
      quote!(),
      a.fields.values().map(|x| match x {
        ImplAgentField::Principal(_) => quote!(()),
        ImplAgentField::Auxiliary(ident) => {
          let (e0, e1) = self.edge_idents(ident);
          let e = if seen.insert(ident) { e0 } else { e1 };
          quote!(#e)
        }
        ImplAgentField::Payload(PayloadPat { pat, .. }) => quote!(#pat),
      }),
    )
  }
}

fn collect_multi_map<K: Ord, V, I: Iterator<Item = (K, V)>>(iter: I) -> BTreeMap<K, Vec<V>> {
  let mut map = BTreeMap::new();
  for (key, val) in iter {
    map.entry(key).or_insert(Vec::new()).push(val);
  }
  map
}
