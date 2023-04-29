use crate::*;
use std::{fmt::Debug, mem::size_of};

#[derive(Clone, Copy)]
pub struct Word(u32);

impl Debug for Word {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.mode() {
      WordMode::Null => write!(f, "Null({:?})", self.as_null_delta().offset_words()),
      WordMode::Kind => write!(f, "Kind({:?})", self.as_kind().id),
      WordMode::Port(mode) => write!(f, "Port({:?}, {:?})", self.as_port().offset_words(), mode),
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
  pub(super) const fn as_port(self) -> Delta {
    debug_assert!(matches!(self.mode(), WordMode::Port(_)));
    Delta {
      offset_bytes: (self.0 & !0b11) as i32,
    }
  }

  #[inline(always)]
  pub(super) const fn as_kind(self) -> Kind {
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
