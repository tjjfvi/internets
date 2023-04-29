use crate::*;
use std::{fmt::Debug, ops::Range};

pub struct ArrayBuffer {
  array: Box<[Word]>,
}

impl Debug for ArrayBuffer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut st = f.debug_map();
    for (i, val) in self.array.iter().enumerate() {
      st.entry(&i, val);
    }
    st.finish()
  }
}

impl Buffer for ArrayBuffer {
  #[inline(always)]
  fn buffer_bounds(&self) -> Range<Addr> {
    let start = unsafe { self.array.get_unchecked(0) } as *const Word as *mut Word;
    let end = unsafe { start.offset(self.array.len() as isize) };
    Addr(start)..Addr(end)
  }

  fn assert_valid(&self, addr: Addr, width: usize) {
    safe! {
      let Range { start, end } = self.buffer_bounds();
      assert!(addr.0 as usize >= start.0 as usize);
      assert!(addr.0 as usize + width <= end.0 as usize);
      assert!(addr.0 as usize & 0b11 == 0);
    }
  }

  fn word(&self, addr: Addr) -> Word {
    self.assert_valid(addr, WORD_SIZE);
    unsafe { *addr.0 }
  }

  fn origin(&self) -> Addr {
    self.buffer_bounds().start
  }

  fn len(&self) -> Delta {
    Delta::of(self.array.len() as i32)
  }
}

impl BufferMut for ArrayBuffer {
  fn word_mut(&mut self, addr: Addr) -> &mut Word {
    self.assert_valid(addr, WORD_SIZE);
    unsafe { &mut *addr.0 }
  }

  fn slice_mut(&mut self, addr: Addr, len: Delta) -> &mut [Word] {
    unsafe { std::slice::from_raw_parts_mut(addr.0, len.offset_bytes as usize / 4) }
  }
}

impl ArrayBuffer {
  pub fn new(size: usize) -> Self {
    ArrayBuffer {
      array: vec![Word::NULL; size].into_boxed_slice(),
    }
  }
}
