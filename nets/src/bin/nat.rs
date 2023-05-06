use internets_nets::*;

#[derive(Clone, Copy, Debug)]
pub struct Nat;

impl Nat {
  pub const ERASE: Kind = Kind::of(0);
  pub const CLONE: Kind = Kind::of(1);
  pub const ZERO: Kind = Kind::of(2);
  pub const SUCC: Kind = Kind::of(3);
  pub const ADD: Kind = Kind::of(4);
  pub const MUL: Kind = Kind::of(5);
}

impl<N: Net> Interactions<N> for Nat {
  #[inline(always)]
  fn reduce(
    &self,
    net: &mut N,
    (a_kind, a_addr): (Kind, Addr),
    (b_kind, b_addr): (Kind, Addr),
  ) -> bool {
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
        net.free(a_addr, Length::of(3));
      }
      (Nat::ERASE, Nat::SUCC) => {
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Kind(Nat::ERASE),
        );
        net.free(b_addr, Length::of(2));
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
        let chunk = net.alloc_write(CHUNK);
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
        net.free(a_addr, Length::of(3));
        net.free(b_addr, Length::of(2));
      }
      (Nat::ZERO, Nat::ADD) => {
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::From(b_addr + Delta::of(2)),
        );
        net.free(b_addr, Length::of(3));
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
        net.free(b_addr, Length::of(3));
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
        let chunk = net.alloc_write(CHUNK);
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
        net.free(a_addr, Length::of(2))
      }
      _ => return false,
    }
    true
  }
}

fn main() {
  let mut stats = Stats::default();
  let buffer = ArrayBuffer::new(1 << 19);
  for _ in 0..1000 {
    let mut net = BasicNet::new(RingAlloc::new(buffer.as_ref()));
    let base = net.alloc_write(&[
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
      Word::kind(Nat::MUL),
      Word::port(Delta::of(4), PortMode::Auxiliary),
      Word::port(Delta::of(-5), PortMode::Principal),
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
      LinkHalf::Port(base + Delta::of(22), PortMode::Principal),
      LinkHalf::Port(base + Delta::of(25), PortMode::Principal),
    );
    reduce_with_stats(&mut net, &Nat, &mut stats);
  }
  println!("{stats}");
}
