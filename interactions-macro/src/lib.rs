mod parser;
use parser::*;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::{format_ident, quote, quote_spanned};
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

  let mut impls = vec![];
  let mut traits = vec![];
  let mut includes = vec![];
  for u in input.items.iter().filter_map(Item::as_use) {
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

  let struct_defs = input
  .items
  .iter()
  .filter_map(Item::as_struct)
  .enumerate()
    .map(|(i, s)| {
      let name = &s.name;
      let vis = &s.vis;
      let i = i as u32;
      let arity_usize = s.parts.iter().filter_map(StructPart::port).count();
      if arity_usize == 0 {
        emit_error!(name.span(), "missing principal port");
        return quote!();
      }
      let principal_idx = s.parts.iter().enumerate().find(|x| x.1.port().is_some()).unwrap().0;
      let arity = arity_usize as u32;
      let parts = s.parts.iter().enumerate().map(|(i, p)| match p {
        StructPart::Port(_) => if i == principal_idx {
          quote!(pub <M as #crate_path::Marker>::Principal<'a>)
        } else {
          quote!(pub <M as #crate_path::Marker>::Auxiliary<'a>)
        },
        StructPart::Payload(PayloadType { ty,..}) => quote!(pub #ty),
      });
      if arity == 1 && s.parts.len() == 1 {
        return quote!(
          #vis struct #name<'a, M: #crate_path::Marker>(#(#parts,)*);
          impl<'a, I: self::Use> #crate_path::GetKind<I> for #name<'a, #crate_path::GetKindMarker> {
            const KIND: #crate_path::Kind = #crate_path::Kind::of(<I as self::Use>::KIND_START + #i);
          }
          impl<'a, I: self::Use> #crate_path::Construct<I> for #name<'a, #crate_path::ConstructMarker> {
            #[inline(always)]
            fn construct<N: #crate_path::Net>(self, net: &mut N, _: &I) {
              let kind = <#name<'a, _> as #crate_path::GetKind<I>>::KIND;
              *self.0 = #crate_path::LinkHalf::Kind(kind);
            }
          }
          impl<'a> #crate_path::Destruct for #name<'a, #crate_path::DestructMarker> {
            #[inline(always)]
            fn destruct<N: #crate_path::Net>(net: &mut N, addr: #crate_path::Addr) -> Self {
              #name(())
            }
            #[inline(always)]
            fn free<N: #crate_path::Net>(_: &mut N, _: #crate_path::Addr) {}
          }
        );
      }
      let payload_len_sum = s
        .parts
        .iter()
        .filter_map(StructPart::payload)
        .map(|PayloadType { ty, .. }| quote!(.add(#crate_path::Length::of_payload::<#ty>())))
        .collect::<Vec<_>>();
      let len = quote!(#crate_path::Length::of(#arity) #(#payload_len_sum)*);
      let destruct_vars = (0..s.parts.len()).map(|i| format_ident!("_{}", i)).collect::<Vec<_>>();
      let ports = s.parts.iter().enumerate().filter_map(|(i, x)| Some((i, x.port()?))).enumerate();
      let payloads = s.parts.iter().enumerate().filter_map(|(i, x)| Some((i, x.payload()?))).enumerate();
      let construct_ports = ports.clone().map(|(delta, (idx, _))| {
        let delta = delta as i32;
        let idx = syn::Index::from(idx);
        let mode = if delta == 0 {
          quote!(Principal)
        } else {
          quote!(Auxiliary)
        };
        quote!(
          *self.#idx = #crate_path::LinkHalf::Port(addr + #crate_path::Delta::of(#delta), #crate_path::PortMode::#mode);
        )
      });
      let construct_payloads = payloads.clone().map(|(payload_i, (idx, PayloadType { ty, ..}))| {
        let prev_payload_len_sum = &payload_len_sum[0..payload_i];
        let idx = syn::Index::from(idx);
        quote!(
          #crate_path::BufferMut::write_payload::<#ty>(net, addr + #crate_path::Length::of(#arity) #(#prev_payload_len_sum)*, self.#idx);
        )
      });
      let destruct_ports = ports.map(|(delta, (idx, _))| {
        let delta = delta as i32;
        let var = &destruct_vars[idx];
        if delta == 0 {
          quote!(
            #var = ();
          )
        } else {
          quote!(
            #var = #crate_path::LinkHalf::From(addr + #crate_path::Delta::of(#delta));
          )
        }
      });
      let destruct_payloads = payloads.map(|(payload_i, (idx, PayloadType { ty, ..}))| {
        let prev_payload_len_sum = &payload_len_sum[0..payload_i];
        let var = &destruct_vars[idx];
        quote!(
          #var = #crate_path::Buffer::read_payload::<#ty>(net, addr + #crate_path::Length::of(#arity) #(#prev_payload_len_sum)*);
        )
      });
      quote!(
        #vis struct #name<'a, M: #crate_path::Marker>(#(#parts,)*);
        impl<'a, I: self::Use> #crate_path::GetKind<I> for #name<'a, #crate_path::GetKindMarker> {
          const KIND: #crate_path::Kind = #crate_path::Kind::of(<I as self::Use>::KIND_START + #i);
        }
        impl<'a, I: self::Use> #crate_path::Construct<I> for #name<'a, #crate_path::ConstructMarker> {
          #[inline(always)]
          fn construct<N: #crate_path::Net>(self, net: &mut N, _: &I) {
            let addr = #crate_path::Alloc::alloc(net, #len);
            let kind = <#name<'a, _> as #crate_path::GetKind<I>>::KIND;
            *#crate_path::BufferMut::word_mut(net, addr) = #crate_path::Word::kind(kind);
            #(#construct_ports)*
            #(#construct_payloads)*
          }
        }
        impl<'a> #crate_path::Destruct for #name<'a, #crate_path::DestructMarker> {
          #[inline(always)]
          fn destruct<N: #crate_path::Net>(net: &mut N, addr: #crate_path::Addr) -> Self {
            #(let #destruct_vars;)*
            #(#destruct_ports)*
            #(#destruct_payloads)*
            #name(#(#destruct_vars),*)
          }
          #[inline(always)]
          fn free<N: #crate_path::Net>(net: &mut N, addr: #crate_path::Addr) {
            #crate_path::Alloc::free(net, addr, #len);
          }
        }
      )
    })
    .collect::<Vec<_>>();

  let fn_defs = input.items.iter().filter_map(Item::as_fn).map(|f| {
    let parts = f.parts.iter().map(|FnPart { ty: p, .. }| match p {
      StructPart::Port(_) => quote!(pub &'a mut #crate_path::LinkHalf),
      StructPart::Payload(PayloadType { ty, .. }) => quote!(pub #ty),
    });
    let binds = f.parts.iter().map(|FnPart { name, ty: p }| {
      let (_, e1) = edge_idents(name);
      match p {
        StructPart::Port(_) => quote!(#e1),
        StructPart::Payload(_) => quote!(#name),
      }
    });
    let sets = f
      .parts
      .iter()
      .filter_map(|x| Some((&x.name, x.ty.port()?)))
      .map(|(name, _)| {
        let (e0, e1) = edge_idents(name);
        quote!(*#e1 = #e0;)
      });
    let lifetime = f
      .parts
      .iter()
      .find(|x| x.ty.port().is_some())
      .map(|_| quote!('a,));
    let mut seen = BTreeSet::new();
    let agents = f
      .net
      .agents
      .iter()
      .map(|x| quote_net_agent(crate_path, quote!(I), quote!(interactions), &mut seen, x))
      .collect::<Vec<_>>();
    let inputs = f.input_idents().collect::<Vec<_>>();
    let links = link_edge_idents(f.inner_idents().filter(|x| !inputs.contains(x)));
    let name = &f.name;
    let vis = &f.vis;
    quote!(
      #[allow(non_camel_case_types)]
      #vis struct #name<#lifetime>(#(#parts),*);
      impl<#lifetime I: self::Use> #crate_path::Construct<I> for #name<#lifetime> {
        #[inline(always)]
        fn construct<N: #crate_path::Net>(self, net: &mut N, interactions: &I) {
          #(#includes)*
          let #name(#(#binds),*) = self;
          #(#agents)*
          #links
          #(#sets)*
        }
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
    let arms = arms
      .into_iter()
      .map(|(a, b, i)| {
        let a_src = quote_src(&a.src);
        let b_src = quote_src(&b.src);
        let a_name = &a.name;
        let b_name = &b.name;
        let mut seen = BTreeSet::new();
        let a_pat = a
          .parts
          .iter()
          .map(|x| match x {
            ImplAgentPart::Principal(_) => quote!(()),
            ImplAgentPart::Auxiliary(ident) => {
              let (e0, e1) = edge_idents(ident);
              let e = if seen.insert(ident) { e0 } else { e1 };
              quote!(#e)
            }
            ImplAgentPart::Payload(PayloadPat { pat, .. }) => quote!(#pat),
          })
          .collect::<Vec<_>>();
        let b_pat = b
          .parts
          .iter()
          .map(|x| match x {
            ImplAgentPart::Principal(_) => quote!(()),
            ImplAgentPart::Auxiliary(ident) => {
              let (e0, e1) = edge_idents(ident);
              let e = if seen.insert(ident) { e0 } else { e1 };
              quote!(#e)
            }
            ImplAgentPart::Payload(PayloadPat { pat, .. }) => quote!(#pat),
          })
          .collect::<Vec<_>>();
        let agents = i
          .net
          .agents
          .iter()
          .map(|x| quote_net_agent(crate_path, quote!(Self), quote!(self), &mut seen, x))
          .collect::<Vec<_>>();
        let links = link_edge_idents(i.all_idents());
        let cond = i.cond.as_ref().map(|x| quote!(if #x)).unwrap_or(quote!());
        let span = i.imp.span;
        quote_spanned!(span=>
          (#a_src #a_name(#(#a_pat),*), #b_src #b_name(#(#b_pat),*)) #cond => {
            #(#agents)*
            #links
          }
        )
      })
      .collect::<Vec<_>>();
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
  })
  .collect::<Vec<_>>();

  let kind_count = struct_defs.len() as u32;

  quote!(
      pub struct Interactions;

      #(#impls)*

      #(#struct_defs)*

      #(#fn_defs)*

      impl self::Use for Interactions {
        const KIND_START: u32 = 0 #(+ <Self as #traits>::KIND_COUNT)*;
      }

      impl<N: #crate_path::Net> #crate_path::Interactions<N> for Interactions {
        #[inline(always)]
        fn reduce(
          &self,
          net: &mut N,
          a: (#crate_path::Kind, #crate_path::Addr),
          b: (#crate_path::Kind, #crate_path::Addr),
        ) -> bool {
          #(#traits::reduce(self, net, a, b) ||)*
          self::Use::reduce(self, net, a, b)
        }
      }

      #[allow(non_upper_case_globals, non_snake_case)]
      pub trait Use: Sized #(+ #traits)* {
        const KIND_START: u32;
        const KIND_COUNT: u32 = #kind_count;
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
    .map(|trait_name| quote!(#trait_name::))
    .unwrap_or(quote!())
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

fn quote_net_agent<'a>(
  crate_path: &TokenStream,
  interactions_ty: TokenStream,
  interactions_var: TokenStream,
  seen: &mut BTreeSet<&'a Ident>,
  agent: &'a NetAgent,
) -> TokenStream {
  let src = quote_src(&agent.src);
  let name = &agent.name;
  let mut vars = vec![];
  let parts = agent
    .parts
    .iter()
    .map(|x| match x {
      NetAgentPart::Port(x) => {
        let (e0, e1) = edge_idents(x);
        let e = if seen.insert(x) { e0 } else { e1 };
        vars.push(e.clone());
        quote!(&mut #e)
      }
      NetAgentPart::Payload(PayloadExpr { expr, .. }) => {
        quote!(#expr)
      }
    })
    .collect::<Vec<_>>();
  quote!(
    #(let mut #vars = #crate_path::LinkHalf::Null;)*
    #crate_path::Construct::<#interactions_ty>::construct(
      #src #name (
        #(#parts),*
      ),
      net,
      #interactions_var,
    );
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
