use std::{
  fmt::Debug,
  mem::size_of,
  ops::{Add, Sub},
};

#[derive(Clone, Copy)]
pub struct Word(u32);

impl Debug for Word {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.unpack().fmt(f)
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortMode {
  Auxiliary = 0,
  Principal = 1,
}

#[derive(Clone, Copy, Debug)]
pub enum UnpackedWord {
  Null,
  Kind(Kind),
  Port(RelAddr, PortMode),
}

pub const WORD_SIZE: usize = 4;
const _WORD_SIZE_IS_FOUR: [u8; WORD_SIZE] = [0; size_of::<Word>()];

impl Word {
  pub const NULL: Word = Word(0);
  #[inline(always)]
  pub fn unpack(self) -> UnpackedWord {
    match self.0 & 0b11 {
      0 => UnpackedWord::Null,
      1 => UnpackedWord::Kind(Kind(self.0 >> 2)),
      _ => UnpackedWord::Port(
        RelAddr((self.0 & !0b11) as i32),
        match self.0 & 0b1 {
          0 => PortMode::Auxiliary,
          _ => PortMode::Principal,
        },
      ),
    }
  }
  pub fn as_kind(self) -> Kind {
    match self.unpack() {
      UnpackedWord::Kind(kind) => kind,
      _ => panic!("expected Kind word, got {:?}", self),
    }
  }
  // pub fn null() -> Self {
  //   Word(0)
  // }
  pub fn kind(kind: Kind) -> Self {
    Word((kind.0 as u32) << 2 | 1)
  }
  pub fn port(rel: RelAddr, mode: PortMode) -> Self {
    Word((rel.0 as u32) | 2 | mode as u32)
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Kind(pub u32);

#[derive(Clone, Copy, Debug)]
pub struct Addr(*mut Word);

impl Addr {
  pub const NULL: Addr = Addr(0 as *mut Word);
}

#[derive(Clone, Copy, Debug)]
pub struct RelAddr(i32);

impl RelAddr {
  pub fn new(idx: i32) -> RelAddr {
    RelAddr(idx * 4)
  }
}

impl Add<RelAddr> for Addr {
  type Output = Addr;
  fn add(self, rel: RelAddr) -> Self::Output {
    Addr(((self.0 as isize) + (rel.0 as isize)) as *mut Word)
  }
}

impl Sub<Addr> for Addr {
  type Output = RelAddr;
  fn sub(self, base: Addr) -> Self::Output {
    RelAddr(((self.0 as isize) - (base.0 as isize)) as i32)
  }
}

#[derive(Debug)]
pub struct Mem {
  buffer: Box<[Word]>,
  alloc_idx: usize,
  pub active: Vec<ActivePair>,
}

impl Mem {
  pub fn new(size: usize) -> Self {
    Mem {
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
  pub fn word(&self, addr: Addr) -> Word {
    self.assert_valid(addr, WORD_SIZE);
    unsafe { *addr.0 }
  }
  pub fn word_mut(&mut self, addr: Addr) -> &mut Word {
    self.assert_valid(addr, WORD_SIZE);
    unsafe { &mut *addr.0 }
  }
  // pub fn u32(&self, addr: Addr) -> u32 {
  //   self.word(addr).0
  // }
  // pub fn u32_mut(&mut self, addr: Addr) -> &mut u32 {
  //   self.assert_valid(addr, 4);
  //   unsafe { &mut *(addr.0 as *mut u32) }
  // }
  // pub fn u64(&self, addr: Addr) -> u64 {
  //   self.assert_valid(addr, 8);
  //   unsafe { *(addr.0 as *const u64) }
  // }
  // pub fn u64_mut(&mut self, addr: Addr) -> &mut u64 {
  //   self.assert_valid(addr, 8);
  //   unsafe { &mut *(addr.0 as *mut u64) }
  // }
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

  pub fn link_opp_opp(&mut self, a: Addr, b: Addr) {
    match self.word(b).unpack() {
      UnpackedWord::Kind(kind) => self.link_opp_nil(a, kind),
      UnpackedWord::Port(rel, PortMode::Auxiliary) => self.link_opp_aux(a, b + rel),
      UnpackedWord::Port(rel, PortMode::Principal) => {
        self.link_opp_prn(a, self.word(b + rel).as_kind(), b + rel)
      }
      _ => unreachable!(),
    }
  }

  pub fn link_opp_nil(&mut self, old: Addr, kind: Kind) {
    let old_port = self.word(old);
    match old_port.unpack() {
      UnpackedWord::Kind(_) => {
        // two nilary agents annihilate
      }
      UnpackedWord::Port(rel, other_mode) => {
        let addr = old + rel;
        if other_mode == PortMode::Auxiliary {
          *self.word_mut(addr) = Word::kind(kind)
        } else {
          self.active.push(ActivePair(
            self.word(addr).as_kind(),
            addr,
            kind,
            Addr::NULL,
          ))
        }
      }
      _ => unreachable!(),
    }
  }

  pub fn link_aux_aux(&mut self, a: Addr, b: Addr) {
    *self.word_mut(a) = Word::port(b - a, PortMode::Auxiliary);
    *self.word_mut(b) = Word::port(a - b, PortMode::Auxiliary);
  }

  pub fn link_aux_prn(&mut self, a: Addr, b: Addr) {
    *self.word_mut(a) = Word::port(b - a, PortMode::Principal);
  }

  pub fn link_opp_aux(&mut self, old: Addr, new: Addr) {
    let old_port = self.word(old);
    match old_port.unpack() {
      UnpackedWord::Kind(_) => *self.word_mut(new) = old_port,
      UnpackedWord::Port(rel, other_mode) => {
        let addr = old + rel;
        *self.word_mut(new) = Word::port(addr - new, other_mode);
        if other_mode == PortMode::Auxiliary {
          *self.word_mut(addr) = Word::port(new - addr, PortMode::Auxiliary);
        }
      }
      _ => unreachable!(),
    }
  }

  pub fn link_opp_prn(&mut self, old: Addr, new_kind: Kind, new: Addr) {
    match self.word(old).unpack() {
      UnpackedWord::Kind(kind) => self
        .active
        .push(ActivePair(kind, Addr::NULL, new_kind, new)),
      UnpackedWord::Port(rel, other_mode) => {
        let addr = old + rel;
        if other_mode == PortMode::Auxiliary {
          *self.word_mut(addr) = Word::port(new - addr, PortMode::Principal);
        } else {
          self
            .active
            .push(ActivePair(self.word(addr).as_kind(), addr, new_kind, new))
        }
      }
      _ => unreachable!(),
    }
  }
}

#[derive(Debug)]
pub struct ActivePair(pub Kind, pub Addr, pub Kind, pub Addr);

pub trait Net {
  fn reduce(&self, mem: &mut Mem, pair: ActivePair);
}
