mod program;
use program::*;

#[derive(Clone, Copy, Debug)]
struct Nat;

impl Nat {
  pub const ERASE: Kind = Kind(0);
  pub const CLONE: Kind = Kind(1);
  pub const ZERO: Kind = Kind(2);
  pub const SUCC: Kind = Kind(3);
  pub const ADD: Kind = Kind(4);
  pub const MUL: Kind = Kind(5);
}

impl Interactions for Nat {
  fn reduce(&self, net: &mut Net, pair: ActivePair) {
    let ((a_kind, a_addr), (b_kind, b_addr)) = net.resolve_active_pair(pair);
    match (a_kind, b_kind) {
      (Nat::ERASE, Nat::ZERO) => {}
      (Nat::CLONE, Nat::ZERO) => {
        net.link(
          LinkHalf::From(a_addr + RelAddr::new(1)),
          LinkHalf::Kind(Nat::ZERO),
        );
        net.link(
          LinkHalf::From(a_addr + RelAddr::new(2)),
          LinkHalf::Kind(Nat::ZERO),
        );
        net.free(a_addr, 3);
      }
      (Nat::ERASE, Nat::SUCC) => {
        net.link(
          LinkHalf::From(b_addr + RelAddr::new(1)),
          LinkHalf::Kind(Nat::ERASE),
        );
        net.free(b_addr, 2);
      }
      (Nat::CLONE, Nat::SUCC) => {
        let chunk = net.alloc(&[
          Word::kind(Nat::SUCC),
          Word::port(RelAddr::new(4), PortMode::Auxiliary),
          Word::kind(Nat::SUCC),
          Word::port(RelAddr::new(3), PortMode::Auxiliary),
          Word::kind(Nat::CLONE),
          Word::port(RelAddr::new(-4), PortMode::Auxiliary),
          Word::port(RelAddr::new(-3), PortMode::Auxiliary),
        ]);
        net.link(
          LinkHalf::From(b_addr + RelAddr::new(1)),
          LinkHalf::Port(chunk + RelAddr::new(4), PortMode::Principal),
        );
        net.link(
          LinkHalf::From(a_addr + RelAddr::new(1)),
          LinkHalf::Port(chunk, PortMode::Principal),
        );
        net.link(
          LinkHalf::From(a_addr + RelAddr::new(2)),
          LinkHalf::Port(chunk + RelAddr::new(2), PortMode::Principal),
        );
        net.free(a_addr, 3);
        net.free(b_addr, 2);
      }
      (Nat::ZERO, Nat::ADD) => {
        net.link(
          LinkHalf::From(b_addr + RelAddr::new(1)),
          LinkHalf::From(b_addr + RelAddr::new(2)),
        );
        net.free(b_addr, 3);
      }
      (Nat::SUCC, Nat::ADD) => {
        let a_pred = a_addr + RelAddr::new(1);
        let b_out = b_addr + RelAddr::new(2);
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
          LinkHalf::From(b_addr + RelAddr::new(1)),
          LinkHalf::Kind(Nat::ERASE),
        );
        net.link(
          LinkHalf::From(b_addr + RelAddr::new(2)),
          LinkHalf::Kind(Nat::ZERO),
        );
        net.free(b_addr, 3);
      }
      (Nat::SUCC, Nat::MUL) => {
        let chunk = net.alloc(&[
          Word::kind(Nat::ADD),
          Word::NULL,
          Word::NULL,
          Word::kind(Nat::CLONE),
          Word::port(RelAddr::new(-4), PortMode::Principal),
          Word::NULL,
        ]);
        net.link(
          LinkHalf::From(b_addr + RelAddr::new(2)),
          LinkHalf::Port(chunk + RelAddr::new(2), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::Port(b_addr + RelAddr::new(2), PortMode::Auxiliary),
          LinkHalf::Port(chunk + RelAddr::new(1), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(b_addr + RelAddr::new(1)),
          LinkHalf::Port(chunk + RelAddr::new(3), PortMode::Principal),
        );
        net.link(
          LinkHalf::Port(b_addr + RelAddr::new(1), PortMode::Auxiliary),
          LinkHalf::Port(chunk + RelAddr::new(5), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(a_addr + RelAddr::new(1)),
          LinkHalf::Port(b_addr, PortMode::Principal),
        );
        net.free(a_addr, 2)
      }
      _ => unimplemented!("{:?} {:?}", a_kind, b_kind),
    }
  }
}

fn main() {
  let mut net = Net::new(1 << 16);
  let base = net.alloc(&[
    Word::port(RelAddr::new(3), PortMode::Auxiliary),
    Word::kind(Nat::MUL),
    Word::port(RelAddr::new(4), PortMode::Auxiliary),
    Word::port(RelAddr::new(-3), PortMode::Auxiliary),
    Word::kind(Nat::CLONE),
    Word::port(RelAddr::new(-4), PortMode::Principal),
    Word::port(RelAddr::new(-4), PortMode::Auxiliary),
    Word::kind(Nat::MUL),
    Word::port(RelAddr::new(4), PortMode::Auxiliary),
    Word::port(RelAddr::new(-5), PortMode::Principal),
    Word::kind(Nat::CLONE),
    Word::port(RelAddr::new(-4), PortMode::Principal),
    Word::port(RelAddr::new(-4), PortMode::Auxiliary),
    Word::kind(Nat::SUCC),
    Word::port(RelAddr::new(1), PortMode::Principal),
    Word::kind(Nat::SUCC),
    Word::kind(Nat::ZERO),
  ]);
  net.link(
    LinkHalf::Port(base + RelAddr::new(10), PortMode::Principal),
    LinkHalf::Port(base + RelAddr::new(13), PortMode::Principal),
  );
  let mut count = 0;
  while let Some(pair) = net.active.pop() {
    // dbg!(&net);
    count += 1;
    Nat.reduce(&mut net, pair);
  }
  dbg!(&net);
  dbg!(count);
}
