use crate::*;

impl Program {
  pub fn compile_structs(&self) -> Vec<TokenStream> {
    self
      .items
      .iter()
      .filter_map(Item::as_struct)
      .enumerate()
      .map(|(i, s)| self.compile_struct(i, s))
      .collect::<Vec<_>>()
  }

  fn compile_struct(&self, i: usize, s: &Struct) -> TokenStream {
    let crate_path = self.crate_path();
    let name = &s.name;
    let vis = &s.vis;
    let i = i as u32;
    let arity_usize = s.parts.iter().filter_map(StructPart::port).count();
    if arity_usize == 0 {
      emit_error!(name.span(), "missing principal port");
      return quote!();
    }
    let principal_idx = s
      .parts
      .iter()
      .enumerate()
      .find(|x| x.1.port().is_some())
      .unwrap()
      .0;
    let arity = arity_usize as u32;
    let parts = s.parts.iter().enumerate().map(|(i, p)| match p {
      StructPart::Port(_) => {
        if i == principal_idx {
          quote!(pub <M as #crate_path::Marker>::Principal<'a>)
        } else {
          quote!(pub <M as #crate_path::Marker>::Auxiliary<'a>)
        }
      }
      StructPart::Payload(PayloadType { ty, .. }) => quote!(pub #ty),
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
    let destruct_vars = (0..s.parts.len())
      .map(|i| format_ident!("_{}", i))
      .collect::<Vec<_>>();
    let ports = s
      .parts
      .iter()
      .enumerate()
      .filter_map(|(i, x)| Some((i, x.port()?)))
      .enumerate();
    let payloads = s
      .parts
      .iter()
      .enumerate()
      .filter_map(|(i, x)| Some((i, x.payload()?)))
      .enumerate();
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
  }
}
