use crate::*;
use std::{
  fmt::{Debug, Display},
  time::{Duration, Instant},
};

pub trait Net: Alloc {
  fn link(&mut self, a: LinkHalf, b: LinkHalf);
  fn reduce(&mut self, interactions: &impl Interactions<Self>) -> bool;
}

#[derive(Debug)]
pub struct BasicNet<M: Alloc> {
  pub mem: M,
  pub active: Vec<ActivePair>,
}

pub enum LinkHalf {
  From(Addr),
  Kind(Kind),
  Port(Addr, PortMode),
}

impl<M: Alloc> DelegateAlloc for BasicNet<M> {
  type Alloc = M;
  #[inline(always)]
  fn delegatee_alloc(&self) -> &Self::Alloc {
    &self.mem
  }
  #[inline(always)]
  fn delegatee_alloc_mut(&mut self) -> &mut Self::Alloc {
    &mut self.mem
  }
}

impl<M: Alloc> Net for BasicNet<M> {
  #[inline(always)]
  fn link(&mut self, a: LinkHalf, b: LinkHalf) {
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
  fn reduce(&mut self, interactions: &impl Interactions<Self>) -> bool {
    if let Some(pair) = self.active.pop() {
      let (a, b) = self.resolve_active_pair(pair);
      interactions.reduce(self, a, b);
      true
    } else {
      false
    }
  }
}

impl<M: Alloc> BasicNet<M> {
  pub fn new(mem: M) -> Self {
    BasicNet {
      mem,
      active: vec![],
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

  #[inline(always)]
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

  #[inline(always)]
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
pub struct ActivePair(pub(super) Word, pub(super) Word);

pub trait Interactions<N: Net + ?Sized> {
  fn reduce(&self, net: &mut N, a: (Kind, Addr), b: (Kind, Addr));
}

#[derive(Default)]
pub struct Stats {
  pub ops: u32,
  pub elapsed: Duration,
}

impl Display for Stats {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let ops = self.ops;
    let elapsed = self.elapsed;
    let speed = (ops as f64) / (elapsed.as_nanos() as f64 / 1.0e3);
    write!(f, "{ops} ops in {elapsed:?} ({speed:.2} op/µs)")
  }
}

#[inline(always)]
pub fn reduce_with_stats<N: Net, I: Interactions<N>>(
  net: &mut N,
  interactions: &I,
  stats: &mut Stats,
) {
  let start = Instant::now();
  let mut ops = 0;
  while net.reduce(interactions) {
    ops += 1;
  }
  stats.elapsed += Instant::now() - start;
  stats.ops += ops;
}
