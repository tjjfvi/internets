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
    let name = &s.name;
    let arity = s.fields.values().filter_map(StructField::port).count() as u32;
    if arity == 0 {
      emit_error!(name.span(), "missing principal port");
      return quote!();
    }
    let struct_def = self.compile_struct_def(s);
    let get_kind_impl = self.compile_get_kind_impl(i, s);
    let construct_destruct_impls = if s.fields.len() == 1 {
      self.compile_nilary_construct_destruct_impls(s)
    } else {
      self.compile_construct_destruct_impls(s, arity)
    };
    quote!(
      #struct_def
      #get_kind_impl
      #construct_destruct_impls
    )
  }

  fn key(&self, s: &Struct, idx: usize) -> TokenStream {
    match &s.fields {
      Fields::Unnamed(_) => {
        let idx = syn::Index::from(idx);
        quote!(#idx)
      }
      Fields::Named(f) => {
        let name = &f.entries[idx].key;
        quote!(#name)
      }
    }
  }

  fn compile_struct_def(&self, s: &Struct) -> TokenStream {
    let crate_path = self.crate_path();
    let name = &s.name;
    let vis = &s.vis;
    let principal_idx = s
      .fields
      .values()
      .enumerate()
      .find(|x| x.1.port().is_some())
      .unwrap()
      .0;
    let fields = self.compile_fields(
      &s.fields,
      quote!(pub),
      s.fields.values().enumerate().map(|(i, p)| match p {
        StructField::Port(_) => {
          if i == principal_idx {
            quote!(<M as #crate_path::Marker>::Principal<'a>)
          } else {
            quote!(<M as #crate_path::Marker>::Auxiliary<'a>)
          }
        }
        StructField::Payload(PayloadType { ty, .. }) => quote!(#ty),
      }),
    );
    let semi = if s.fields.semi() { quote!(;) } else { quote!() };
    quote!(#vis struct #name<'a, M: #crate_path::Marker> #fields #semi)
  }

  fn compile_get_kind_impl(&self, i: usize, s: &Struct) -> TokenStream {
    let crate_path = self.crate_path();
    let name = &s.name;
    let i = i as u32;
    quote!(
      impl<'a, I: self::Use> #crate_path::GetKind<I> for #name<'a, #crate_path::GetKindMarker> {
        const KIND: #crate_path::Kind = #crate_path::Kind::of(<I as self::Use>::KIND_START + #i);
      }
    )
  }

  fn compile_nilary_construct_destruct_impls(&self, s: &Struct) -> TokenStream {
    let crate_path = self.crate_path();
    let name = &s.name;
    quote!(
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
    )
  }

  fn compile_construct_destruct_impls(&self, s: &Struct, arity: u32) -> TokenStream {
    let crate_path = self.crate_path();
    let arity_len = quote!(#crate_path::Length::of(#arity));
    let payload_lens = s
      .fields
      .values()
      .filter_map(StructField::payload)
      .map(|PayloadType { ty, .. }| quote!(#crate_path::Length::of_payload::<#ty>()))
      .collect::<Vec<_>>();
    let len = quote!(#arity_len #(.add(#payload_lens))*);
    let construct_impl = self.compile_construct_impl(s, &len, &arity_len, &payload_lens);
    let destruct_impl = self.compile_destruct_impl(s, &len, &arity_len, &payload_lens);
    quote!(
      #construct_impl
      #destruct_impl
    )
  }

  fn compile_construct_impl(
    &self,
    s: &Struct,
    len: &TokenStream,
    arity_len: &TokenStream,
    payload_lens: &Vec<TokenStream>,
  ) -> TokenStream {
    let crate_path = self.crate_path();
    let name = &s.name;
    let construct_ports = s.ports().map(|(delta, (idx, _))| {
      let delta = delta as i32;
      let key = self.key(s, idx);
      let mode = if delta == 0 {
        quote!(Principal)
      } else {
        quote!(Auxiliary)
      };
      quote!(
        *self.#key = #crate_path::LinkHalf::Port(
          addr + #crate_path::Delta::of(#delta),
          #crate_path::PortMode::#mode,
        );
      )
    });
    let construct_payloads = s
      .payloads()
      .map(|(payload_i, (idx, PayloadType { ty, .. }))| {
        let prev_payload_lens = &payload_lens[0..payload_i];
        let key = self.key(s, idx);
        quote!(
          #crate_path::BufferMut::write_payload::<#ty>(
            net,
            addr + #arity_len #(.add(#prev_payload_lens))*,
            self.#key,
          );
        )
      });
    quote!(
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
    )
  }

  fn compile_destruct_impl(
    &self,
    s: &Struct,
    len: &TokenStream,
    arity_len: &TokenStream,
    payload_lens: &Vec<TokenStream>,
  ) -> TokenStream {
    let crate_path = self.crate_path();
    let name = &s.name;
    let destruct_vars = (0..s.fields.len())
      .map(|i| format_ident!("_{}", i))
      .collect::<Vec<_>>();
    let destruct_ports = s.ports().map(|(delta, (idx, _))| {
      let delta = delta as i32;
      let var = &destruct_vars[idx];
      if delta == 0 {
        quote!(#var = ();)
      } else {
        quote!(#var = #crate_path::LinkHalf::From(addr + #crate_path::Delta::of(#delta));)
      }
    });
    let destruct_payloads = s
      .payloads()
      .map(|(payload_i, (idx, PayloadType { ty, .. }))| {
        let prev_payload_lens = &payload_lens[0..payload_i];
        let var = &destruct_vars[idx];
        quote!(
          #var = #crate_path::Buffer::read_payload::<#ty>(
            net,
            addr + #arity_len #(.add(#prev_payload_lens))*,
          );
        )
      });
    let fields = self.compile_fields(
      &s.fields,
      quote!(),
      destruct_vars.iter().map(|x| quote!(#x)),
    );
    quote!(
      impl<'a> #crate_path::Destruct for #name<'a, #crate_path::DestructMarker> {
        #[inline(always)]
        fn destruct<N: #crate_path::Net>(net: &mut N, addr: #crate_path::Addr) -> Self {
          #(let #destruct_vars;)*
          #(#destruct_ports)*
          #(#destruct_payloads)*
          #name #fields
        }
        #[inline(always)]
        fn free<N: #crate_path::Net>(net: &mut N, addr: #crate_path::Addr) {
          #crate_path::Alloc::free(net, addr, #len);
        }
      }
    )
  }
}
