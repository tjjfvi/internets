mod fields;
mod fns;
mod impls;
mod net;
mod structs;
mod uses;

pub use fields::*;
pub use fns::*;
pub use impls::*;
pub use net::*;
pub use structs::*;
pub use uses::*;

use crate::*;

impl Program {
  pub fn compile(&self) -> TokenStream {
    let crate_path = &self.crate_path();

    let (impls, includes, traits) = self.compile_uses();
    let struct_defs = self.compile_structs();
    let fn_defs = self.compile_fns(&includes);
    let rules = self.compile_impls();

    let kind_count = struct_defs.len() as u32;

    quote!(
        pub struct Interactions;

        #impls
        #(#struct_defs)*
        #fn_defs

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
            #includes
            match (a_kind, b_kind) {
              #(#rules)*
              _ if false => {},
              _ => return false,
            }
            #[allow(unreachable_code)]
            true
          }
        }
    )
  }

  pub fn crate_path(&self) -> TokenStream {
    quote!(::internets_nets)
  }
}
