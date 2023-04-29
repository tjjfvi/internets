use crate::*;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Net<M: Alloc> {
  pub(super) mem: M,
  pub(super) active: Vec<ActivePair>,
}

pub enum LinkHalf {
  From(Addr),
  Kind(Kind),
  Port(Addr, PortMode),
}

impl<M: Alloc> DelegateAlloc for Net<M> {
  type Alloc = M;
  fn delegatee_alloc(&self) -> &Self::Alloc {
    &self.mem
  }
  fn delegatee_alloc_mut(&mut self) -> &mut Self::Alloc {
    &mut self.mem
  }
}

impl<M: Alloc> Net<M> {
  pub fn new(mem: M) -> Self {
    Net {
      mem,
      active: vec![],
    }
  }

  #[inline(always)]
  pub fn link(&mut self, a: LinkHalf, b: LinkHalf) {
    let a = self.get_link_half(a);
    let b = self.get_link_half(b);
    use LinkHalf::*;
    use PortMode::*;
    match (a, b) {
      (Port(a, Auxiliary), Port(b, Auxiliary)) => self.link_aux_aux(a, b),
      (Port(a, Auxiliary), Port(b, Principal)) => self.link_aux_prn(a, b),
      (Port(a, Auxiliary), Kind(b)) => self.link_aux_nil(a, b),
      (Port(a, Principal), Port(b, Auxiliary)) => self.link_aux_prn(b, a),
      (Port(a, Principal), Port(b, Principal)) => self.link_prn_prn(a, b),
      (Port(a, Principal), Kind(b)) => self.link_prn_nil(a, b),
      (Kind(a), Port(b, Auxiliary)) => self.link_aux_nil(b, a),
      (Kind(a), Port(b, Principal)) => self.link_prn_nil(b, a),
      (Kind(a), Kind(b)) => self.link_nil_nil(a, b),
      _ => fail!(unreachable!()),
    }
  }

  #[inline(always)]
  fn get_link_half(&self, link_half: LinkHalf) -> LinkHalf {
    match link_half {
      LinkHalf::From(addr) => {
        let word = self.word(addr);
        match word.mode() {
          WordMode::Kind => LinkHalf::Kind(word.as_kind()),
          WordMode::Port(mode) => LinkHalf::Port(addr + word.as_port(), mode),
          _ => fail!(unreachable!()),
        }
      }
      x => x,
    }
  }

  fn link_aux_aux(&mut self, a: Addr, b: Addr) {
    *self.word_mut(a) = Word::port(b - a, PortMode::Auxiliary);
    *self.word_mut(b) = Word::port(a - b, PortMode::Auxiliary);
  }

  fn link_aux_prn(&mut self, a: Addr, b: Addr) {
    *self.word_mut(a) = Word::port(b - a, PortMode::Principal);
  }

  fn link_aux_nil(&mut self, a: Addr, b: Kind) {
    *self.word_mut(a) = Word::kind(b)
  }

  fn link_prn_prn(&mut self, a: Addr, b: Addr) {
    self.active.push(ActivePair(
      Word::port(a - self.origin(), PortMode::Principal),
      Word::port(b - self.origin(), PortMode::Principal),
    ));
  }

  fn link_prn_nil(&mut self, a: Addr, b: Kind) {
    self.active.push(ActivePair(
      Word::port(a - self.origin(), PortMode::Principal),
      Word::kind(b),
    ));
  }

  fn link_nil_nil(&mut self, _a: Kind, _b: Kind) {
    // they just annihilate
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
}

#[derive(Debug)]
pub(super) struct ActivePair(pub(super) Word, pub(super) Word);

pub trait Interactions {
  fn reduce<M: Alloc>(&self, net: &mut Net<M>, a: (Kind, Addr), b: (Kind, Addr));
}
