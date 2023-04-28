use crate::*;
use std::{fmt::Debug, ops::Range};

pub struct Net {
  pub(super) buffer: Box<[Word]>,
  pub(super) alloc: Addr,
  pub(super) active: Vec<ActivePair>,
}

impl Debug for Net {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut st = f.debug_struct("Net");
    st.field("buffer", &DebugBuffer(&*self.buffer));
    st.field("alloc", &(self.alloc - self.origin()));
    st.field("active", &self.active);
    return st.finish();

    struct DebugBuffer<'a>(&'a [Word]);
    impl<'a> Debug for DebugBuffer<'a> {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut st = f.debug_map();
        for (i, val) in self.0.iter().enumerate() {
          st.entry(&i, val);
        }
        st.finish()
      }
    }
  }
}

impl Net {
  pub(super) fn buffer_bounds(&self) -> Range<usize> {
    let start = (&self.buffer[0]) as *const Word as usize;
    let end = start + self.buffer.len() * WORD_SIZE;
    start..end
  }

  pub(super) fn assert_valid(&self, addr: Addr, width: usize) {
    let Range { start, end } = self.buffer_bounds();
    assert!(addr.0 as usize >= start);
    assert!(addr.0 as usize + width <= end);
    assert!(addr.0 as usize & 0b11 == 0);
  }

  pub(super) fn word(&self, addr: Addr) -> Word {
    self.assert_valid(addr, WORD_SIZE);
    unsafe { *addr.0 }
  }

  pub(super) fn word_mut(&mut self, addr: Addr) -> &mut Word {
    self.assert_valid(addr, WORD_SIZE);
    unsafe { &mut *addr.0 }
  }

  pub(super) fn origin(&self) -> Addr {
    Addr(&self.buffer[0] as *const Word as *mut Word)
  }
}

#[derive(Debug)]
pub(super) struct ActivePair(pub(super) Word, pub(super) Word);
