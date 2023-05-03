mod parser;
use parser::*;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote, ToTokens};
use std::collections::{BTreeMap, BTreeSet};
use syn::{parse_macro_input, Ident, Path};

#[proc_macro_error]
#[proc_macro]
pub fn interactions(input: TokenStream1) -> TokenStream1 {
  _interactions(input.into()).into()
}

fn _interactions(input: TokenStream1) -> TokenStream1 {
  let input = parse_macro_input!(input as Input);

  ensure_unique(
    input
      .items
      .iter()
      .filter_map(Item::as_struct)
      .map(|x| &x.name),
  );

  for item in &input.items {
    match item {
      Item::Struct(_) => {}
      Item::Impl(i) => ensure_used_twice(i.all_idents()),
      Item::Fn(f) => ensure_used_twice(f.all_idents()),
    }
  }

  let kinds = input
    .items
    .iter()
    .filter_map(Item::as_struct)
    .enumerate()
    .map(|(i, s)| (&s.name, (i, s)))
    .collect::<BTreeMap<_, _>>();
  let crate_path = &input.crate_path;
  let ty_name = &input.ty;

  let struct_defs = input
  .items
  .iter()
  .filter_map(Item::as_struct)
  .enumerate()
    .map(|(i, s)| {
      let name = &s.name;
      let kind_ident = make_kind_ident(name);
      let len_ident = make_len_ident(name);
      let arity_ident = make_arity_ident(name);
      let i = i as u32;
      let arity_usize = s.ports.len();
      let arity = arity_usize as u32;
      if arity == 1 && s.payload.is_none() {
        return quote!(
          pub const #kind_ident: #crate_path::Kind = #crate_path::Kind::of(#i);
          pub const #len_ident: #crate_path::Length = #crate_path::Length::of(0);
          pub const #arity_ident: #crate_path::Length = #crate_path::Length::of(0);
          pub fn #name<N: #crate_path::Net>(_: &mut N)
            -> [#crate_path::LinkHalf; 1]
          {
            [#crate_path::LinkHalf::Kind(#ty_name::#kind_ident)]
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
      let (payload_arg, payload_set) = s.payload.as_ref().as_ref().map(|x| {
        let ty = &x.ty;
        (
        quote!(payload: #ty),
        quote!(#crate_path::BufferMut::write_payload::<#ty>(net, chunk + #ty_name::#arity_ident, payload);),
        )
      }).unwrap_or((quote!(), quote!()));
      quote!(
        pub const #kind_ident: #crate_path::Kind = #crate_path::Kind::of(#i);
        pub const #len_ident: #crate_path::Length = #crate_path::Length::of(#arity) #payload_add;
        pub const #arity_ident: #crate_path::Length = #crate_path::Length::of(#arity);
        #[inline(always)]
        pub fn #name<N: #crate_path::Net>(net: &mut N, #payload_arg)
          -> [#crate_path::LinkHalf; #arity_usize]
        {
          let chunk = #crate_path::Alloc::alloc(net, #ty_name::#len_ident);
          *#crate_path::BufferMut::word_mut(net, chunk) = #crate_path::Word::kind(#ty_name::#kind_ident);
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
      .map(|x| quote_net_agent(ty_name, x))
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
      pub fn #name<N: #crate_path::Net>(net: &mut N)
        -> [#crate_path::LinkHalf; #arity_usize]
      {
        #decls
        #(#agents)*
        #links
        [#(#returns),*]
      }
    )
  });

  let rules = collect_multi_map(input.items.iter().filter_map(Item::as_impl).map(|i| {
    let (a, b) = if kinds.get(&i.left.name).unwrap().0 < kinds.get(&i.right.name).unwrap().0 {
      (&i.left, &i.right)
    } else {
      (&i.right, &i.left)
    };
    ((&a.name, &b.name), (a, b, i))
  }))
  .into_iter()
  .map(|((a_name, b_name), arms)| {
    let a_kind = make_kind_ident(a_name);
    let b_kind = make_kind_ident(b_name);
    let a_len = make_len_ident(a_name);
    let b_len = make_len_ident(b_name);
    let a_arity = make_arity_ident(a_name);
    let b_arity = make_arity_ident(b_name);
    let a_payload_read = kinds[a_name]
      .1
      .payload
      .as_ref()
      .map(|x| {
        let ty = &x.ty;
        quote!(#crate_path::Buffer::read_payload::<#ty>(net, a_addr + #ty_name::#a_arity))
      })
      .unwrap_or(quote!(()));
    let b_payload_read = kinds[b_name]
      .1
      .payload
      .as_ref()
      .map(|x| {
        let ty = &x.ty;
        quote!(#crate_path::Buffer::read_payload::<#ty>(net, b_addr + #ty_name::#b_arity))
      })
      .unwrap_or(quote!(()));
    let arms = arms
      .into_iter()
      .map(|(a, b, i)| {
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
          .map(|x| quote_net_agent(ty_name, x))
          .collect::<Vec<_>>();
        let decls = declare_edge_idents(i.all_idents());
        let links = link_edge_idents(i.all_idents());
        let cond = i.cond.as_ref().map(|x| quote!(if #x)).unwrap_or(quote!());
        quote!(
          (#a_pat, #b_pat) #cond => {
            #decls
            #a_binds
            #b_binds
            #(#agents)*
            #links
            if #ty_name::#a_len.non_zero() {
              #crate_path::Alloc::free(net, a_addr, #ty_name::#a_len);
            }
            if #ty_name::#b_len.non_zero() {
              #crate_path::Alloc::free(net, b_addr, #ty_name::#b_len);
            }
          }
        )
      })
      .collect::<Vec<_>>();
    quote!(
      (#ty_name::#a_kind, #ty_name::#b_kind) => {
        match (#a_payload_read, #b_payload_read) {
          #(#arms),*
        }
      }
    )
  })
  .collect::<Vec<_>>();

  quote!(
    struct #ty_name;

    #[allow(non_upper_case_globals, non_snake_case)]
    impl #ty_name {
      #(#struct_defs)*
      #(#fn_defs)*
    }

    impl<N: #crate_path::Net> #crate_path::Interactions<N> for #ty_name {
      #[inline(always)]
      fn reduce(
        &self,
        net: &mut N,
        (a_kind, a_addr): (#crate_path::Kind, #crate_path::Addr),
        (b_kind, b_addr): (#crate_path::Kind, #crate_path::Addr),
      ) {
        match (a_kind, b_kind) {
          #(#rules)*
          _ => unimplemented!("{:?}, {:?}", a_kind, b_kind),
        }
      }
    }
  )
  .into()
}

fn destructure_agent(crate_path: &Path, addr: TokenStream, agent: &ImplAgent) -> TokenStream {
  let links = (1..=agent.aux.len() as i32)
    .map(|i| quote!(#crate_path::LinkHalf::From(#addr + #crate_path::Delta::of(#i))))
    .collect::<Vec<_>>();
  let binds = bind_ports(&agent.aux, quote!([#(#links),*]));
  binds
}

fn make_kind_ident(struct_name: &Ident) -> Ident {
  format_ident!("{}_KIND", struct_name)
}

fn make_arity_ident(struct_name: &Ident) -> Ident {
  format_ident!("{}_ARITY", struct_name)
}

fn make_len_ident(struct_name: &Ident) -> Ident {
  format_ident!("{}_LEN", struct_name)
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
        abort!(ident.span(), "identifier used more than twice")
      }
    } else {
      once.insert(ident);
    }
  }
  for ident in once {
    abort!(ident.span(), "identifier used only once")
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

fn quote_net_agent(ty_name: &Ident, agent: &NetAgent) -> TokenStream {
  let name = &agent.name;
  let payload_arg = agent
    .payload
    .as_ref()
    .map(|x| {
      let expr = &x.expr;
      quote!(#expr)
    })
    .unwrap_or(quote!());
  bind_ports(&agent.ports, quote!(#ty_name::#name(net, #payload_arg)))
}

fn bind_ports(ports: &Vec<Ident>, source: TokenStream) -> TokenStream {
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
  quote!(
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
