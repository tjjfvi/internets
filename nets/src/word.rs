use crate::*;
use std::{
  fmt::Debug,
  mem::size_of,
  sync::atomic::{AtomicU32, Ordering},
};

#[derive(Clone, Copy)]
pub struct Word(pub u32);

impl Debug for Word {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.mode() {
      WordMode::Null => write!(
        f,
        "{:08x} = Null({:?})",
        self.0,
        self.as_null_delta().offset_words()
      ),
      WordMode::Kind => write!(f, "{:08x} = Kind({:?})", self.0, self.as_kind().id),
      WordMode::Port(mode) => write!(
        f,
        "{:08x} = Port({:?}, {:?})",
        self.0,
        self.as_port().offset_words(),
        mode
      ),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortMode {
  Auxiliary = 0,
  Principal = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WordMode {
  Null,
  Kind,
  Port(PortMode),
}

pub const WORD_SIZE: usize = 4;
const _WORD_SIZE_IS_FOUR: [u8; WORD_SIZE] = [0; size_of::<Word>()];

impl Word {
  #[inline(always)]
  pub(super) const fn mode(self) -> WordMode {
    match self.0 & 0b11 {
      0 => WordMode::Null,
      1 => WordMode::Kind,
      2 | 3 => WordMode::Port(match self.0 & 0b1 {
        0 => PortMode::Auxiliary,
        1 => PortMode::Principal,
        _ => fail!(unreachable!()),
      }),
      _ => fail!(unreachable!()),
    }
  }

  #[inline(always)]
  pub(super) const fn as_null_len(self) -> Length {
    debug_assert!(matches!(self.mode(), WordMode::Null));
    Length {
      length_bytes: self.0,
    }
  }

  #[inline(always)]
  pub(super) const fn as_null_delta(self) -> Delta {
    debug_assert!(matches!(self.mode(), WordMode::Null));
    Delta {
      offset_bytes: self.0 as i32,
    }
  }

  #[inline(always)]
  pub const fn as_port(self) -> Delta {
    debug_assert!(matches!(self.mode(), WordMode::Port(_)));
    Delta {
      offset_bytes: (self.0 & !0b11) as i32,
    }
  }

  #[inline(always)]
  pub const fn as_kind(self) -> Kind {
    debug_assert!(matches!(self.mode(), WordMode::Kind));
    Kind {
      id: (self.0 >> 2) as u32,
    }
  }
}

impl Word {
  pub const NULL: Word = Word(0);

  #[inline(always)]
  pub(super) const fn null_len(len: Length) -> Self {
    Word(len.length_bytes)
  }

  #[inline(always)]
  pub(super) const fn null_delta(delta: Delta) -> Self {
    Word(delta.offset_bytes as u32)
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

// #[derive(Debug)]
// pub struct AtomicWord(pub AtomicU32);

// impl AtomicWord {
//   #[inline(always)]
//   pub fn new(&self, value: Word) -> AtomicWord {
//     AtomicWord(AtomicU32::new(value.0))
//   }
//   #[inline(always)]
//   pub fn read(&self, order: Ordering) -> Word {
//     Word(self.0.load(order))
//   }
//   #[inline(always)]
//   pub fn write(&mut self, value: Word, order: Ordering) {
//     self.0.store(value.0, order)
//   }
//   #[inline(always)]
//   pub fn swap(&mut self, val: Word, order: Ordering) -> Word {
//     Word(self.0.swap(val.0, order))
//   }
//   #[inline(always)]
//   pub fn compare_exchange_weak(
//     &mut self,
//     current: Word,
//     new: Word,
//     success: Ordering,
//     failure: Ordering,
//   ) -> Result<Word, Word> {
//     match self
//       .0
//       .compare_exchange_weak(current.0, new.0, success, failure)
//     {
//       Ok(x) => Ok(Word(x)),
//       Err(x) => Err(Word(x)),
//     }
//   }
// }

#[derive(Debug)]
pub struct AtomicWord(Word);

impl AtomicWord {
  #[inline(always)]
  pub fn new(&self, value: Word) -> AtomicWord {
    AtomicWord(value)
  }
  #[inline(always)]
  pub fn read(&self, _: Ordering) -> Word {
    self.0
  }
  #[inline(always)]
  pub fn write(&mut self, value: Word, _: Ordering) {
    self.0 = value
  }
  #[inline(always)]
  pub fn swap(&mut self, val: Word, _: Ordering) -> Word {
    std::mem::replace(&mut self.0, val)
  }
  #[inline(always)]
  pub fn compare_exchange_weak(
    &mut self,
    current: Word,
    new: Word,
    _: Ordering,
    _: Ordering,
  ) -> Result<Word, Word> {
    self.0 = new;
    Ok(current)
  }
}
