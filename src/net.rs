use std::{
  fmt::Debug,
  mem::size_of,
  ops::{Add, Sub},
};

#[derive(Clone, Copy)]
pub struct Word(u32);

impl Debug for Word {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.mode() {
      WordMode::Null => write!(f, "Null"),
      WordMode::Kind => write!(f, "Kind({:?})", self.as_kind().id),
      WordMode::Port(mode) => write!(f, "Port({:?}, {:?})", self.as_port().offset_bytes / 4, mode),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortMode {
  Auxiliary = 0,
  Principal = 1,
}

#[derive(Clone, Copy, Debug)]
enum WordMode {
  Null,
  Kind,
  Port(PortMode),
}

pub const WORD_SIZE: usize = 4;
const _WORD_SIZE_IS_FOUR: [u8; WORD_SIZE] = [0; size_of::<Word>()];

impl Word {
  pub const NULL: Word = Word(0);
  #[inline(always)]
  const fn mode(self) -> WordMode {
    match self.0 & 0b11 {
      0 => WordMode::Null,
      1 => WordMode::Kind,
      2 | 3 => WordMode::Port(match self.0 & 0b1 {
        0 => PortMode::Auxiliary,
        1 => PortMode::Principal,
        _ => unreachable!(),
      }),
      _ => unreachable!(),
    }
  }
  #[inline(always)]
  const fn as_port(self) -> Delta {
    debug_assert!(matches!(self.mode(), WordMode::Port(_)));
    Delta {
      offset_bytes: (self.0 & !0b11) as i32,
    }
  }
  #[inline(always)]
  const fn as_kind(self) -> Kind {
    debug_assert!(matches!(self.mode(), WordMode::Kind));
    Kind {
      id: (self.0 >> 2) as u32,
    }
  }
  #[inline(always)]
  pub const fn kind(kind: Kind) -> Self {
    Word((kind.id as u32) << 2 | 1)
  }
  #[inline(always)]
  pub const fn port(delta: Delta, mode: PortMode) -> Self {
    Word((delta.offset_bytes as u32) | 2 | mode as u32)
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Kind {
  pub id: u32,
}

impl Kind {
  pub const fn of(id: u32) -> Kind {
    Kind { id }
  }
}

#[derive(Clone, Copy, Debug)]
pub struct Addr(*mut Word);

impl Addr {
  pub const NULL: Addr = Addr(0 as *mut Word);
}

#[derive(Clone, Copy, Debug)]
pub struct Delta {
  offset_bytes: i32,
}

impl Delta {
  #[inline(always)]
  pub const fn of(delta: i32) -> Delta {
    Delta {
      offset_bytes: delta * (WORD_SIZE as i32),
    }
  }
}

impl Add<Delta> for Addr {
  type Output = Addr;
  #[inline(always)]
  fn add(self, delta: Delta) -> Self::Output {
    Addr(((self.0 as isize) + (delta.offset_bytes as isize)) as *mut Word)
  }
}

impl Sub<Addr> for Addr {
  type Output = Delta;
  #[inline(always)]
  fn sub(self, base: Addr) -> Self::Output {
    Delta {
      offset_bytes: ((self.0 as isize) - (base.0 as isize)) as i32,
    }
  }
}

#[derive(Debug)]
pub struct Net {
  buffer: Box<[Word]>,
  alloc_idx: usize,
  active: Vec<ActivePair>,
}

pub enum LinkHalf {
  From(Addr),
  Kind(Kind),
  Port(Addr, PortMode),
}

impl Net {
  pub fn new(size: usize) -> Self {
    Net {
      buffer: vec![Word::NULL; size].into_boxed_slice(),
      alloc_idx: 0,
      active: Vec::new(),
    }
  }

  fn assert_valid(&self, addr: Addr, width: usize) {
    let buffer_start = (&self.buffer[0]) as *const Word as usize;
    let buffer_end = buffer_start + self.buffer.len() * WORD_SIZE;
    assert!(addr.0 as usize >= buffer_start);
    assert!(addr.0 as usize + width <= buffer_end);
    assert!(addr.0 as usize & 0b11 == 0);
  }

  fn word(&self, addr: Addr) -> Word {
    self.assert_valid(addr, WORD_SIZE);
    unsafe { *addr.0 }
  }

  fn word_mut(&mut self, addr: Addr) -> &mut Word {
    self.assert_valid(addr, WORD_SIZE);
    unsafe { &mut *addr.0 }
  }

  pub fn alloc(&mut self, data: &[Word]) -> Addr {
    let old_idx = self.alloc_idx;
    self.alloc_idx += data.len();
    self.buffer[old_idx..self.alloc_idx].copy_from_slice(data);
    Addr(&mut self.buffer[old_idx] as *mut Word)
  }

  pub fn free(&mut self, addr: Addr, len: usize) {
    self.assert_valid(addr, len * WORD_SIZE);
    unsafe { std::slice::from_raw_parts_mut(addr.0, len) }.fill(Word::NULL);
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
  fn get_link_half(&self, link_half: LinkHalf) -> LinkHalf {
    match link_half {
      LinkHalf::From(addr) => {
        let word = self.word(addr);
        match word.mode() {
          WordMode::Kind => LinkHalf::Kind(word.as_kind()),
          WordMode::Port(mode) => LinkHalf::Port(addr + word.as_port(), mode),
          _ => unreachable!(),
        }
      }
      x => x,
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
      _ => unreachable!(),
    }
  }

  fn origin(&self) -> Addr {
    Addr(&self.buffer[0] as *const Word as *mut Word)
  }

  fn resolve_active_half(&self, word: Word) -> (Kind, Addr) {
    match word.mode() {
      WordMode::Kind => (word.as_kind(), Addr::NULL),
      WordMode::Port(PortMode::Principal) => {
        let addr = self.origin() + word.as_port();
        (self.word(addr).as_kind(), addr)
      }
      _ => unreachable!(),
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

#[derive(Debug)]
struct ActivePair(Word, Word);

pub trait Interactions {
  fn reduce(&self, net: &mut Net, a: (Kind, Addr), b: (Kind, Addr));
}
