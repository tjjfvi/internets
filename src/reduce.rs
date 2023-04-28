use crate::*;

impl Net {
  fn resolve_active_half(&self, word: Word) -> (Kind, Addr) {
    match word.mode() {
      WordMode::Kind => (word.as_kind(), Addr::NULL),
      WordMode::Port(PortMode::Principal) => {
        let addr = self.origin() + word.as_port();
        (self.word(addr).as_kind(), addr)
      }
      _ => fail!(unreachable!()),
    }
  }

  fn resolve_active_pair(&self, pair: ActivePair) -> ((Kind, Addr), (Kind, Addr)) {
    let a = self.resolve_active_half(pair.0);
    let b = self.resolve_active_half(pair.1);
    if a.0 > b.0 {
      (b, a)
    } else {
      (a, b)
    }
  }

  pub fn reduce(&mut self, interactions: &impl Interactions) -> bool {
    if let Some(pair) = self.active.pop() {
      let (a, b) = self.resolve_active_pair(pair);
      interactions.reduce(self, a, b);
      true
    } else {
      false
    }
  }
}

pub trait Interactions {
  fn reduce(&self, net: &mut Net, a: (Kind, Addr), b: (Kind, Addr));
}
