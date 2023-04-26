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
}

impl Net for Nat {
  fn reduce(&self, mem: &mut Mem, pair: ActivePair) {
    let ActivePair(a_kind, a_addr, b_kind, b_addr) = if pair.0 > pair.2 {
      ActivePair(pair.2, pair.3, pair.0, pair.1)
    } else {
      pair
    };
    match (a_kind, b_kind) {
      (Nat::ERASE, Nat::ZERO) => {}
      (Nat::CLONE, Nat::ZERO) => {
        mem.relink_nullary(a_addr + RelAddr::new(1), Nat::ZERO);
        mem.relink_nullary(a_addr + RelAddr::new(2), Nat::ZERO);
        mem.free(a_addr, 3);
      }
      (Nat::ERASE, Nat::SUCC) => {
        mem.relink_nullary(b_addr + RelAddr::new(1), Nat::ERASE);
        mem.free(b_addr, 2);
      }
      (Nat::CLONE, Nat::SUCC) => {
        let chunk = mem.alloc(&[
          Word::kind(Nat::SUCC),
          Word::port(RelAddr::new(4), PortMode::Auxiliary),
          Word::kind(Nat::SUCC),
          Word::port(RelAddr::new(3), PortMode::Auxiliary),
          Word::kind(Nat::CLONE),
          Word::port(RelAddr::new(-4), PortMode::Auxiliary),
          Word::port(RelAddr::new(-3), PortMode::Auxiliary),
        ]);
        mem.relink_principal(
          b_addr + RelAddr::new(1),
          Nat::CLONE,
          chunk + RelAddr::new(4),
        );
        mem.relink_principal(a_addr + RelAddr::new(1), Nat::SUCC, chunk);
        mem.relink_principal(a_addr + RelAddr::new(2), Nat::SUCC, chunk + RelAddr::new(2));
        mem.free(a_addr, 3);
        mem.free(b_addr, 2);
      }
      (Nat::ZERO, Nat::ADD) => {
        mem.relink(b_addr + RelAddr::new(1), b_addr + RelAddr::new(2));
        mem.free(b_addr, 3);
      }
      (Nat::SUCC, Nat::ADD) => {
        let a_pred = a_addr + RelAddr::new(1);
        let b_out = b_addr + RelAddr::new(2);
        mem.relink_principal(b_out, Nat::SUCC, a_addr);
        mem.relink_principal(a_pred, Nat::ADD, b_addr);
        mem.link_auxillary(a_pred, b_out);
      }
      _ => unimplemented!(),
    }
  }
}

fn main() {
  let mut mem = Mem::new(64);
  let base = mem.alloc(&[
    Word::port(RelAddr::new(3), PortMode::Auxiliary),
    Word::kind(Nat::ADD),
    Word::port(RelAddr::new(4), PortMode::Auxiliary),
    Word::port(RelAddr::new(-3), PortMode::Auxiliary),
    Word::kind(Nat::CLONE),
    Word::port(RelAddr::new(-4), PortMode::Principal),
    Word::port(RelAddr::new(-4), PortMode::Auxiliary),
    Word::kind(Nat::ADD),
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
  mem.active.push(ActivePair(
    Nat::CLONE,
    base + RelAddr::new(10),
    Nat::SUCC,
    base + RelAddr::new(13),
  ));
  dbg!(&mem);
  let mut count = 0;
  while let Some(pair) = mem.active.pop() {
    count += 1;
    Nat.reduce(&mut mem, pair);
    dbg!(&mem);
  }
  dbg!(count);
}
