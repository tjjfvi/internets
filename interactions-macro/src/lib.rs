mod parser;
use parser::*;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use std::collections::{BTreeMap, BTreeSet};
use syn::{spanned::Spanned, Ident};

#[proc_macro_error]
#[proc_macro]
pub fn interactions(input: TokenStream1) -> TokenStream1 {
  _interactions(input.into()).into()
}

fn _interactions(input: TokenStream1) -> TokenStream {
  let input = match syn::parse::<Input>(input) {
    Ok(data) => data,
    Err(err) => {
      return TokenStream::from(err.to_compile_error());
    }
  };

  let crate_path = &quote!(::internets_nets);

  ensure_unique(
    input
      .items
      .iter()
      .filter_map(Item::as_struct)
      .map(|x| &x.name),
  );

  for item in &input.items {
    match item {
      Item::Impl(i) => ensure_used_twice(i.all_idents()),
      Item::Fn(f) => ensure_used_twice(f.all_idents()),
      _ => {}
    }
  }

  let ty_name = &input.ty;
  let trait_name = make_trait_name(ty_name);
  let trait_name = quote!(#trait_name);
  let ty_vis = &input.vis;
  let self_as_trait = quote!(<Self as #trait_name>);

  let mut impls = vec![];
  let mut traits = vec![];
  let mut includes = vec![quote!(#[allow(unused)] use #trait_name as #ty_name;)];
  for u in input.items.iter().filter_map(Item::as_use) {
    let path = &u.path;
    let use_ty_name = &path.segments.last().unwrap().ident;
    let path = {
      let trait_name = make_trait_name(use_ty_name);
      let leading = path.leading_colon;
      let segments = path.segments.iter().rev().skip(1).rev().collect::<Vec<_>>();
      quote!(#leading #(#segments::)* #trait_name)
    };
    impls.push(quote!(
      impl #path for #ty_name {
        const KIND_START: u32 = 0 #(+ <Self as #traits>::KIND_COUNT)*;
      }
    ));
    let span = path.span();
    includes.push(quote_spanned!(span=>
      #[allow(unused)]
      use #path as #use_ty_name;
    ));
    traits.push(path);
  }

  let struct_defs = input
  .items
  .iter()
  .filter_map(Item::as_struct)
  .enumerate()
    .map(|(i, s)| {
      let name = &s.name;
      let payload_ident = make_payload_ident(name);
      let kind_ident = make_kind_ident(name);
      let len_ident = make_len_ident(name);
      let arity_ident = make_arity_ident(name);
      let i = i as u32;
      let arity_usize = s.ports.len();
      let arity = arity_usize as u32;
      if arity == 1 && s.payload.is_none() {
        return quote!(
          const #kind_ident: #crate_path::Kind = #crate_path::Kind::of(#self_as_trait::KIND_START + #i);
          const #len_ident: #crate_path::Length = #crate_path::Length::of(0);
          const #arity_ident: #crate_path::Length = #crate_path::Length::of(0);
          fn #payload_ident<N: #crate_path::Net>(_: &mut N, _: #crate_path::Addr) {}
          fn #name<N: #crate_path::Net>(_: &mut N)
            -> [#crate_path::LinkHalf; 1]
          {
            [#crate_path::LinkHalf::Kind(#self_as_trait::#kind_ident)]
          }
        );
      }
      let payload_add = s
        .payload
        .as_ref()
        .map(|x| &x.ty)
        .map(|ty| quote!(.add(#crate_path::Length::of_payload::<#ty>())))
        .unwrap_or(quote!());
      let ports = s
        .ports
        .iter()
        .enumerate()
        .map(|(i, _)| {
          if i == 0 {
            quote!(#crate_path::LinkHalf::Port(chunk, #crate_path::PortMode::Principal))
          } else {
            let i = i as i32;
            quote!(#crate_path::LinkHalf::Port(
              chunk + #crate_path::Delta::of(#i),
              #crate_path::PortMode::Auxiliary,
            ))
          }
        })
        .collect::<Vec<_>>();
      let (read_payload, payload_arg, payload_set) = s.payload.as_ref().map(|x| {
        let ty = &x.ty;
        (
          quote!(fn #payload_ident<N: #crate_path::Net>(net: &mut N, addr: #crate_path::Addr) -> #ty{
            #crate_path::Buffer::read_payload::<#ty>(net, addr + #self_as_trait::#arity_ident)
          }),
        quote!(payload: #ty),
        quote!(#crate_path::BufferMut::write_payload::<#ty>(net, chunk + #self_as_trait::#arity_ident, payload);),
        )
      }).unwrap_or((quote!(
        fn #payload_ident<N: #crate_path::Net>(_: &mut N, _: #crate_path::Addr) {}
      ), quote!(), quote!()));
      quote!(
        const #kind_ident: #crate_path::Kind = #crate_path::Kind::of(#self_as_trait::KIND_START + #i);
        const #len_ident: #crate_path::Length = #crate_path::Length::of(#arity) #payload_add;
        const #arity_ident: #crate_path::Length = #crate_path::Length::of(#arity);
        #read_payload
        #[inline(always)]
        fn #name<N: #crate_path::Net>(net: &mut N, #payload_arg)
          -> [#crate_path::LinkHalf; #arity_usize]
        {
          let chunk = #crate_path::Alloc::alloc(net, #self_as_trait::#len_ident);
          *#crate_path::BufferMut::word_mut(net, chunk) = #crate_path::Word::kind(#self_as_trait::#kind_ident);
          #payload_set
          [#(#ports),*]
        }
      )
    })
    .collect::<Vec<_>>();

  let fn_defs = input.items.iter().filter_map(Item::as_fn).map(|f| {
    let agents = f
      .net
      .agents
      .iter()
      .map(|x| quote_net_agent(x))
      .collect::<Vec<_>>();
    let decls = declare_edge_idents(f.all_idents());
    let links = link_edge_idents(f.all_idents().filter(|x| !f.inputs.contains(x)));
    let name = &f.name;
    let arity_usize = f.inputs.len();
    let returns = f
      .inputs
      .iter()
      .map(|ident| {
        let (id_0, _) = edge_idents(ident);
        quote!(#id_0)
      })
      .collect::<Vec<_>>();
    quote!(
      fn #name<N: #crate_path::Net>(net: &mut N)
        -> [#crate_path::LinkHalf; #arity_usize]
      {
        #(#includes)*
        #decls
        #(#agents)*
        #links
        [#(#returns),*]
      }
    )
  });

  let rules = collect_multi_map(input.items.iter().filter_map(Item::as_impl).flat_map(|i| {
    let a = &i.left;
    let b = &i.right;
    [
      ((&a.src, &a.name, &b.src, &b.name), (a, b, i)),
      ((&b.src, &b.name, &a.src, &a.name), (b, a, i)),
    ]
  }))
  .into_iter()
  .map(|((a_src, a_name, b_src, b_name), arms)| {
    let a_src = quote_src(&a_src);
    let b_src = quote_src(&b_src);
    let a_kind = make_kind_ident(a_name);
    let b_kind = make_kind_ident(b_name);
    let a_len = make_len_ident(a_name);
    let b_len = make_len_ident(b_name);
    let a_payload = make_payload_ident(a_name);
    let b_payload = make_payload_ident(b_name);
    let arms = arms
      .into_iter()
      .map(|(a, b, i)| {
        let a_src = quote_src(&a.src);
        let b_src = quote_src(&b.src);
        let a_name = &a.name;
        let b_name = &b.name;
        let a_pat = a
          .payload
          .as_ref()
          .map(|p| p.pat.to_token_stream())
          .unwrap_or(quote!(()));
        let b_pat = b
          .payload
          .as_ref()
          .map(|p| p.pat.to_token_stream())
          .unwrap_or(quote!(()));
        let a_binds = destructure_agent(crate_path, quote!(a_addr), a);
        let b_binds = destructure_agent(crate_path, quote!(b_addr), b);
        let agents = i
          .net
          .agents
          .iter()
          .map(|x| quote_net_agent(x))
          .collect::<Vec<_>>();
        let decls = declare_edge_idents(i.all_idents());
        let links = link_edge_idents(i.all_idents());
        let cond = i.cond.as_ref().map(|x| quote!(if #x)).unwrap_or(quote!());
        let span = i.imp.span;
        quote_spanned!(span=>
          (#a_pat, #b_pat) #cond => {
            let _ = #a_src::#a_name::<N>;
            let _ = #b_src::#b_name::<N>;
            #decls
            #a_binds
            #b_binds
            #(#agents)*
            #links
          }
        )
      })
      .collect::<Vec<_>>();
    let a_kind_path = quote!(#a_src::#a_kind);
    let b_kind_path = quote!(#b_src::#b_kind);
    quote!(
      x if (#a_kind_path <= #b_kind_path) && x == (#a_kind_path, #b_kind_path) => {
        match (
          #a_src::#a_payload(net, a_addr),
          #b_src::#b_payload(net, b_addr),
        ) {
          #(#arms),*
        }
        if #a_src::#a_len.non_zero() {
          #crate_path::Alloc::free(net, a_addr, #a_src::#a_len);
        }
        if #b_src::#b_len.non_zero() {
          #crate_path::Alloc::free(net, b_addr, #b_src::#b_len);
        }
      }
    )
  })
  .collect::<Vec<_>>();

  let kind_count = struct_defs.len() as u32;

  quote!(
    #ty_vis struct #ty_name;

    #(#impls)*

    impl #trait_name for #ty_name {
      const KIND_START: u32 = 0 #(+ <Self as #traits>::KIND_COUNT)*;
    }

    impl<N: #crate_path::Net> #crate_path::Interactions<N> for #ty_name {
      #[inline(always)]
      fn reduce(
        &self,
        net: &mut N,
        a: (#crate_path::Kind, #crate_path::Addr),
        b: (#crate_path::Kind, #crate_path::Addr),
      ) -> bool {
        #(#traits::reduce(self, net, a, b) ||)*
        #trait_name::reduce(self, net, a, b)
      }
    }

    #[allow(non_upper_case_globals, non_snake_case)]
    #ty_vis trait #trait_name: #(#traits),* {
      const KIND_START: u32;
      const KIND_COUNT: u32 = #kind_count;
      #(#struct_defs)*
      #(#fn_defs)*
      #[inline(always)]
      fn reduce<N: #crate_path::Net>(
        &self,
        net: &mut N,
        (a_kind, a_addr): (#crate_path::Kind, #crate_path::Addr),
        (b_kind, b_addr): (#crate_path::Kind, #crate_path::Addr),
      ) -> bool {
        #(#includes)*
        match (a_kind, b_kind) {
          #(#rules)*
          _ => return false,
        }
        true
      }
    }
  )
}

fn quote_src(src: &Option<Ident>) -> TokenStream {
  src
    .as_ref()
    .map(|trait_name| quote_spanned!(trait_name.span()=> <Self as #trait_name>))
    .unwrap_or(quote!(Self))
}

fn destructure_agent(
  crate_path: &TokenStream,
  addr: TokenStream,
  agent: &ImplAgent,
) -> TokenStream {
  let links = (1..=agent.aux.len() as i32)
    .map(|i| quote!(#crate_path::LinkHalf::From(#addr + #crate_path::Delta::of(#i))))
    .collect::<Vec<_>>();
  let binds = bind_ports(agent.name.span(), &agent.aux, quote!([#(#links),*]));
  binds
}

fn make_trait_name(ty_name: &Ident) -> Ident {
  format_ident!("Use{}", ty_name)
}

fn make_payload_ident(struct_name: &Ident) -> Ident {
  format_ident!("{}_read_payload", struct_name, span = Span::mixed_site())
}

fn make_kind_ident(struct_name: &Ident) -> Ident {
  format_ident!("{}_KIND", struct_name, span = Span::mixed_site())
}

fn make_arity_ident(struct_name: &Ident) -> Ident {
  format_ident!("{}_ARITY", struct_name, span = Span::mixed_site())
}

fn make_len_ident(struct_name: &Ident) -> Ident {
  format_ident!("{}_LEN", struct_name, span = Span::mixed_site())
}

fn ensure_unique<'a, I: Iterator<Item = &'a Ident>>(idents: I) {
  let mut seen = BTreeSet::new();
  for ident in idents {
    if !seen.insert(ident) {
      abort!(ident.span(), "duplicate identifier")
    }
  }
}

fn ensure_used_twice<'a, I: Iterator<Item = &'a Ident>>(idents: I) {
  let mut all = BTreeSet::new();
  let mut once = BTreeSet::new();
  for ident in idents {
    if !all.insert(ident) {
      if !once.remove(ident) {
        emit_error!(ident.span(), "identifier used more than twice");
      }
    } else {
      once.insert(ident);
    }
  }
  for ident in once {
    emit_error!(ident.span(), "identifier used only once");
  }
}

fn declare_edge_idents<'a, I: Iterator<Item = &'a Ident>>(idents: I) -> TokenStream {
  let decls = idents
    .collect::<BTreeSet<_>>()
    .iter()
    .map(|ident| {
      let (id_0, _) = edge_idents(ident);
      quote!(let #id_0 = ();)
    })
    .collect::<Vec<_>>();
  quote!(#(#decls)*)
}

fn link_edge_idents<'a, I: Iterator<Item = &'a Ident>>(idents: I) -> TokenStream {
  let links = idents
    .collect::<BTreeSet<_>>()
    .iter()
    .map(|ident| {
      let (id_0, id_1) = edge_idents(ident);
      quote!(net.link(#id_0, #id_1);)
    })
    .collect::<Vec<_>>();
  quote!(#(#links)*)
}

fn quote_net_agent(agent: &NetAgent) -> TokenStream {
  let src = quote_src(&agent.src);
  let name = &agent.name;
  let payload_arg = agent
    .payload
    .as_ref()
    .map(|x| {
      let expr = &x.expr;
      quote!(#expr)
    })
    .unwrap_or(quote!());
  bind_ports(
    agent.name.span(),
    &agent.ports,
    quote!(#src::#name(net, #payload_arg)),
  )
}

fn bind_ports(span: Span, ports: &Vec<Ident>, source: TokenStream) -> TokenStream {
  if ports.is_empty() {
    return quote!();
  }
  let binds = ports
    .iter()
    .enumerate()
    .map(|(i, _)| {
      let temp_ident = format_ident!("_temp_{}", i);
      quote!(#temp_ident)
    })
    .collect::<Vec<_>>();
  let shifts = ports
    .iter()
    .enumerate()
    .map(|(i, port)| {
      let (id_0, id_1) = edge_idents(port);
      let temp_ident = &binds[i];
      quote!(
        #[allow(unused_variables)]
        let #id_1 = #id_0;
        let #id_0 = #temp_ident;
      )
    })
    .collect::<Vec<_>>();
  quote_spanned!(span=>
    let [#(#binds),*] = #source;
    #(#shifts)*
  )
}

fn edge_idents(ident: &Ident) -> (Ident, Ident) {
  (format_ident!("{}_0", ident), format_ident!("{}_1", ident))
}

fn collect_multi_map<K: Ord, V, I: Iterator<Item = (K, V)>>(iter: I) -> BTreeMap<K, Vec<V>> {
  let mut map = BTreeMap::new();
  for (key, val) in iter {
    map.entry(key).or_insert(Vec::new()).push(val);
  }
  map
}
