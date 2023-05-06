use crate::*;
use std::{
  fmt::{Debug, Display},
  sync::atomic::Ordering,
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

#[derive(Clone, Copy)]
pub enum LinkHalf {
  From(Addr),
  Kind(Kind),
  Port(Addr, PortMode),
}

impl<M: Alloc> DelegateAlloc for BasicNet<M> {
  type Alloc = M;
  #[inline(always)]
  fn alloc(&self) -> &Self::Alloc {
    &self.mem
  }
  #[inline(always)]
  fn alloc_mut(&mut self) -> &mut Self::Alloc {
    &mut self.mem
  }
}

impl<M: Alloc> Net for BasicNet<M> {
  #[inline(always)]
  fn link(&mut self, a: LinkHalf, b: LinkHalf) {
    self.link(a, b)
  }

  #[inline(always)]
  fn reduce(&mut self, interactions: &impl Interactions<Self>) -> bool {
    if let Some(pair) = self.active.pop() {
      let (a, b) = self.resolve_active_pair(pair);
      let did_reduce = interactions.reduce(self, a, b);
      debug_assert!(did_reduce);
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
  fn match_link<T>(
    &mut self,
    a: LinkHalf,
    aux_new: impl Fn(Addr) -> Option<Word>,
    aux_cont: impl Fn(&mut Self, Addr) -> T,
    prn: impl Fn(&mut Self, Addr) -> T,
    nil: impl Fn(&mut Self, Kind) -> T,
  ) -> T {
    // let a = self.get_link_half(a);
    match a {
      LinkHalf::From(a) => self._match_link(a, aux_new, aux_cont, prn, nil),
      LinkHalf::Port(b, PortMode::Auxiliary) => {
        if let Some(w) = aux_new(b) {
          self.word(b).write(w, Ordering::Relaxed);
        }
        aux_cont(self, b)
      }
      LinkHalf::Port(b, PortMode::Principal) => prn(self, b),
      LinkHalf::Kind(b) => nil(self, b),
    }
  }

  // #[inline(never)]
  // fn _match_link<T>(
  //   &mut self,
  //   a: Addr,
  //   aux_new: impl Fn(Addr) -> Word,
  //   aux_cont: impl Fn(&mut Self, Addr) -> T,
  //   prn: impl Fn(&mut Self, Addr) -> T,
  //   nil: impl Fn(&mut Self, Kind) -> T,
  // ) -> T {
  //   loop {
  //     let a_word = self.word(a).swap(Word::NULL, Ordering::Relaxed);
  //     let b = match a_word.mode() {
  //       WordMode::Kind => return nil(self, a_word.as_kind()),
  //       WordMode::Port(PortMode::Principal) => return prn(self, a + a_word.as_port()),
  //       WordMode::Null => {
  //         std::hint::spin_loop();
  //         continue;
  //       }
  //       WordMode::Port(PortMode::Auxiliary) => {
  //         fence(Ordering::Acquire);
  //         a + a_word.as_port()
  //       }
  //     };
  //     match self.word(b).compare_exchange_weak(
  //       Word::port(a - b, PortMode::Auxiliary),
  //       aux_new(b),
  //       Ordering::Release,
  //       Ordering::Relaxed,
  //     ) {
  //       Ok(_) => return aux_cont(self, b),
  //       Err(_) => {
  //         self.word(a).write(a_word, Ordering::Relaxed);
  //         std::hint::spin_loop();
  //       }
  //     }
  //   }
  // }

  #[inline(always)]
  fn _match_link<T>(
    &mut self,
    a: Addr,
    aux_new: impl Fn(Addr) -> Option<Word>,
    aux_cont: impl Fn(&mut Self, Addr) -> T,
    prn: impl Fn(&mut Self, Addr) -> T,
    nil: impl Fn(&mut Self, Kind) -> T,
  ) -> T {
    // let a_word = self.word(a).swap(Word::NULL, Ordering::Relaxed);
    // let a_word = self.word(a).read(Ordering::Relaxed);
    let a_word = self.read_word(a);
    match a_word.mode() {
      WordMode::Kind => return nil(self, a_word.as_kind()),
      WordMode::Port(PortMode::Principal) => return prn(self, a + a_word.as_port()),
      WordMode::Port(PortMode::Auxiliary) => {
        // fence(Ordering::Acquire);
        let b = a + a_word.as_port();
        if let Some(w) = aux_new(b) {
          self.word(b).write(w, Ordering::Relaxed);
        }
        aux_cont(self, b)
      }
      WordMode::Null => {
        fail!(unreachable!());
      }
    }
  }

  #[inline(always)]
  pub fn link(&mut self, a: LinkHalf, b: LinkHalf) {
    self.match_link(
      a,
      #[inline(always)]
      |_| None,
      #[inline(always)]
      |slf, a| slf.link_aux(b, a),
      #[inline(always)]
      |slf, a| slf.link_prn(b, a),
      #[inline(always)]
      |slf, a| slf.link_nil(b, a),
    )
  }

  #[inline(always)]
  fn link_aux(&mut self, b: LinkHalf, a: Addr) {
    self.match_link(
      b,
      #[inline(always)]
      |b| Some(Word::port(a - b, PortMode::Auxiliary)),
      #[inline(always)]
      |slf, b| {
        slf
          .word(a)
          .write(Word::port(b - a, PortMode::Auxiliary), Ordering::Relaxed)
      },
      #[inline(always)]
      |slf, b| slf.link_aux_prn(a, b),
      #[inline(always)]
      |slf, b| slf.link_aux_nil(a, b),
    )
  }

  #[inline(always)]
  fn link_prn(&mut self, b: LinkHalf, a: Addr) {
    self.match_link(
      b,
      #[inline(always)]
      |b| Some(Word::port(a - b, PortMode::Principal)),
      #[inline(always)]
      |_, _| {},
      #[inline(always)]
      |slf, b| slf.link_prn_prn(a, b),
      #[inline(always)]
      |slf, b| slf.link_prn_nil(a, b),
    )
  }

  #[inline(always)]
  fn link_nil(&mut self, b: LinkHalf, a: Kind) {
    self.match_link(
      b,
      #[inline(always)]
      |_| Some(Word::kind(a)),
      #[inline(always)]
      |_, _| {},
      #[inline(always)]
      |slf, b| slf.link_prn_nil(b, a),
      #[inline(always)]
      |slf, b| slf.link_nil_nil(a, b),
    )
  }

  #[inline(always)]
  fn link_aux_prn(&mut self, a: Addr, b: Addr) {
    self
      .word(a)
      .write(Word::port(b - a, PortMode::Principal), Ordering::Relaxed);
  }

  #[inline(always)]
  fn link_aux_nil(&mut self, a: Addr, b: Kind) {
    self.word(a).write(Word::kind(b), Ordering::Relaxed)
  }

  #[inline(always)]
  fn link_prn_prn(&mut self, a: Addr, b: Addr) {
    self.active.push(ActivePair(
      Word::port(a - self.origin(), PortMode::Principal),
      Word::port(b - self.origin(), PortMode::Principal),
    ));
  }

  #[inline(always)]
  fn link_prn_nil(&mut self, a: Addr, b: Kind) {
    self.active.push(ActivePair(
      Word::port(a - self.origin(), PortMode::Principal),
      Word::kind(b),
    ));
  }

  #[inline(always)]
  fn link_nil_nil(&mut self, _a: Kind, _b: Kind) {
    // they just annihilate
  }

  #[inline(always)]
  fn resolve_active_half(&self, word: Word) -> (Kind, Addr) {
    match word.mode() {
      WordMode::Kind => (word.as_kind(), Addr::NULL),
      WordMode::Port(PortMode::Principal) => {
        let addr = self.origin() + word.as_port();
        (self.read_word(addr).as_kind(), addr)
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

/*

loop {
  b = a.swap(NULL, Acquire)
  if b.c_e_w(a, n, Release, Relaxed) {
    n.store(b, Relaxed)
    break;
  } else {
    a.store(b, Relaxed)
    continue;
  }
}

 */

/*

loop {
  x = a.swap(NULL, Acquire)
  if x.c_e_w(a, NULL, Relaxed, Relaxed) {
    break;
  } else {
    a.store(x, Relaxed)
    continue;
  }
}



*/

#[derive(Debug)]
pub struct ActivePair(pub(super) Word, pub(super) Word);

pub trait Interactions<N: Net + ?Sized> {
  fn reduce(&self, net: &mut N, a: (Kind, Addr), b: (Kind, Addr)) -> bool;
}

pub trait InteractionsMod<N: Net + ?Sized>: Interactions<N> {
  type Shift<const KIND_START: u32>: InteractionsMod<N>;
  const KIND_END: u32;
}

#[derive(Default)]
pub struct Stats {
  pub ops: u64,
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
