mod addr;
mod alloc;
mod delta;
mod kind;
mod link;
mod net;
mod reduce;
mod word;
pub use addr::*;
pub use alloc::*;
pub use delta::*;
pub use kind::*;
pub use link::*;
pub use net::*;
pub use reduce::*;
pub use word::*;

#[derive(Clone, Copy, Debug)]
struct Nat;

impl Nat {
  pub const ERASE: Kind = Kind::of(0);
  pub const CLONE: Kind = Kind::of(1);
  pub const ZERO: Kind = Kind::of(2);
  pub const SUCC: Kind = Kind::of(3);
  pub const ADD: Kind = Kind::of(4);
  pub const MUL: Kind = Kind::of(5);
}

impl Interactions for Nat {
  fn reduce(&self, net: &mut Net, (a_kind, a_addr): (Kind, Addr), (b_kind, b_addr): (Kind, Addr)) {
    match (a_kind, b_kind) {
      (Nat::ERASE, Nat::ZERO) => {}
      (Nat::CLONE, Nat::ZERO) => {
        net.link(
          LinkHalf::From(a_addr + Delta::of(1)),
          LinkHalf::Kind(Nat::ZERO),
        );
        net.link(
          LinkHalf::From(a_addr + Delta::of(2)),
          LinkHalf::Kind(Nat::ZERO),
        );
        net.free(a_addr, Delta::of(3));
      }
      (Nat::ERASE, Nat::SUCC) => {
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Kind(Nat::ERASE),
        );
        net.free(b_addr, Delta::of(2));
      }
      (Nat::CLONE, Nat::SUCC) => {
        const CHUNK: &'static [Word] = &[
          Word::kind(Nat::SUCC),
          Word::port(Delta::of(4), PortMode::Auxiliary),
          Word::kind(Nat::SUCC),
          Word::port(Delta::of(3), PortMode::Auxiliary),
          Word::kind(Nat::CLONE),
          Word::port(Delta::of(-4), PortMode::Auxiliary),
          Word::port(Delta::of(-3), PortMode::Auxiliary),
        ];
        let chunk = net.alloc(CHUNK);
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Port(chunk + Delta::of(4), PortMode::Principal),
        );
        net.link(
          LinkHalf::From(a_addr + Delta::of(1)),
          LinkHalf::Port(chunk, PortMode::Principal),
        );
        net.link(
          LinkHalf::From(a_addr + Delta::of(2)),
          LinkHalf::Port(chunk + Delta::of(2), PortMode::Principal),
        );
        net.free(a_addr, Delta::of(3));
        net.free(b_addr, Delta::of(2));
      }
      (Nat::ZERO, Nat::ADD) => {
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::From(b_addr + Delta::of(2)),
        );
        net.free(b_addr, Delta::of(3));
      }
      (Nat::SUCC, Nat::ADD) => {
        let a_pred = a_addr + Delta::of(1);
        let b_out = b_addr + Delta::of(2);
        net.link(
          LinkHalf::From(b_out),
          LinkHalf::Port(a_addr, PortMode::Principal),
        );
        net.link(
          LinkHalf::From(a_pred),
          LinkHalf::Port(b_addr, PortMode::Principal),
        );
        net.link(
          LinkHalf::Port(a_pred, PortMode::Auxiliary),
          LinkHalf::Port(b_out, PortMode::Auxiliary),
        );
      }
      (Nat::ZERO, Nat::MUL) => {
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Kind(Nat::ERASE),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(2)),
          LinkHalf::Kind(Nat::ZERO),
        );
        net.free(b_addr, Delta::of(3));
      }
      (Nat::SUCC, Nat::MUL) => {
        const CHUNK: &'static [Word] = &[
          Word::kind(Nat::ADD),
          Word::NULL,
          Word::NULL,
          Word::kind(Nat::CLONE),
          Word::port(Delta::of(-4), PortMode::Principal),
          Word::NULL,
        ];
        let chunk = net.alloc(CHUNK);
        net.link(
          LinkHalf::From(b_addr + Delta::of(2)),
          LinkHalf::Port(chunk + Delta::of(2), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::Port(b_addr + Delta::of(2), PortMode::Auxiliary),
          LinkHalf::Port(chunk + Delta::of(1), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Port(chunk + Delta::of(3), PortMode::Principal),
        );
        net.link(
          LinkHalf::Port(b_addr + Delta::of(1), PortMode::Auxiliary),
          LinkHalf::Port(chunk + Delta::of(5), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(a_addr + Delta::of(1)),
          LinkHalf::Port(b_addr, PortMode::Principal),
        );
        net.free(a_addr, Delta::of(2))
      }
      _ => unimplemented!("{:?} {:?}", a_kind, b_kind),
    }
  }
}

fn main() {
  let mut net = Net::new(1 << 8);
  let base = net.alloc(&[
    Word::port(Delta::of(3), PortMode::Auxiliary),
    Word::kind(Nat::MUL),
    Word::port(Delta::of(4), PortMode::Auxiliary),
    Word::port(Delta::of(-3), PortMode::Auxiliary),
    Word::kind(Nat::CLONE),
    Word::port(Delta::of(-4), PortMode::Principal),
    Word::port(Delta::of(-4), PortMode::Auxiliary),
    Word::kind(Nat::MUL),
    Word::port(Delta::of(4), PortMode::Auxiliary),
    Word::port(Delta::of(-5), PortMode::Principal),
    Word::kind(Nat::CLONE),
    Word::port(Delta::of(-4), PortMode::Principal),
    Word::port(Delta::of(-4), PortMode::Auxiliary),
    Word::kind(Nat::SUCC),
    Word::port(Delta::of(1), PortMode::Principal),
    Word::kind(Nat::SUCC),
    Word::kind(Nat::ZERO),
  ]);
  net.link(
    LinkHalf::Port(base + Delta::of(10), PortMode::Principal),
    LinkHalf::Port(base + Delta::of(13), PortMode::Principal),
  );
  let mut count = 0;
  while net.reduce(&Nat) {
    count += 1;
  }
  dbg!(&net);
  dbg!(count);
}
