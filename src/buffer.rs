use std::{
  fmt::Debug,
  ops::{Deref, DerefMut, Range},
};

use crate::*;

pub trait Buffer {
  fn buffer_bounds(&self) -> Range<Addr>;
  fn assert_valid(&self, addr: Addr, width: usize);

  fn word(&self, addr: Addr) -> Word;

  fn origin(&self) -> Addr;
  fn len(&self) -> Delta;
}

pub trait BufferMut: Buffer {
  fn word_mut(&mut self, addr: Addr) -> &mut Word;

  fn slice_mut(&mut self, addr: Addr, len: Delta) -> &mut [Word];
}

pub struct ArrayBuffer(pub Box<[Word]>);

impl Debug for ArrayBuffer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut st = f.debug_map();
    for (i, val) in self.0.iter().enumerate() {
      st.entry(&i, val);
    }
    st.finish()
  }
}

impl Buffer for ArrayBuffer {
  #[inline(always)]
  fn buffer_bounds(&self) -> Range<Addr> {
    let start = unsafe { self.0.get_unchecked(0) } as *const Word as *mut Word;
    let end = unsafe { start.offset(self.0.len() as isize) };
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
    Delta::of(self.0.len() as i32)
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

impl<'a, T: Deref> Buffer for T
where
  <T as Deref>::Target: Buffer,
{
  fn buffer_bounds(&self) -> Range<Addr> {
    (&**self).buffer_bounds()
  }
  fn assert_valid(&self, addr: Addr, width: usize) {
    (&**self).assert_valid(addr, width)
  }
  fn word(&self, addr: Addr) -> Word {
    (&**self).word(addr)
  }
  fn origin(&self) -> Addr {
    (&**self).origin()
  }
  fn len(&self) -> Delta {
    (&**self).len()
  }
}

impl<'a, T: DerefMut> BufferMut for T
where
  <T as Deref>::Target: BufferMut,
{
  fn word_mut(&mut self, addr: Addr) -> &mut Word {
    (&mut **self).word_mut(addr)
  }

  fn slice_mut(&mut self, addr: Addr, len: Delta) -> &mut [Word] {
    (&mut **self).slice_mut(addr, len)
  }
}
