use crate::*;
use std::{
  fmt::Debug,
  mem::size_of,
  ops::Range,
  ptr::{read_unaligned, write_unaligned},
};

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

  #[inline(always)]
  fn assert_valid(&self, addr: Addr, length: Length) {
    safe! {
      let Range { start, end } = self.buffer_bounds();
      assert!(addr.0 as usize >= start.0 as usize);
      assert!(addr.0 as usize + length.length_bytes as usize <= end.0 as usize);
      assert!(addr.0 as usize & 0b11 == 0);
    }
  }

  #[inline(always)]
  fn word(&self, addr: Addr) -> Word {
    self.assert_valid(addr, Length::of(1));
    unsafe { *addr.0 }
  }

  #[inline(always)]
  fn read_payload<P>(&self, addr: Addr) -> P {
    self.assert_valid(addr, Length::of(((size_of::<P>() + 1) >> 2) as u32));
    unsafe { read_unaligned(addr.0 as *mut P) }
  }

  #[inline(always)]
  fn origin(&self) -> Addr {
    self.buffer_bounds().start
  }

  #[inline(always)]
  fn len(&self) -> Length {
    Length::of(self.array.len() as u32)
  }
}

impl BufferMut for ArrayBuffer {
  #[inline(always)]
  fn word_mut(&mut self, addr: Addr) -> &mut Word {
    self.assert_valid(addr, Length::of(1));
    unsafe { &mut *addr.0 }
  }

  #[inline(always)]
  fn write_payload<P>(&mut self, addr: Addr, value: P) {
    self.assert_valid(addr, Length::of(((size_of::<P>() + 1) >> 2) as u32));
    unsafe { write_unaligned(addr.0 as *mut P, value) }
  }

  #[inline(always)]
  fn slice_mut(&mut self, addr: Addr, len: Length) -> &mut [Word] {
    unsafe { std::slice::from_raw_parts_mut(addr.0, len.length_words() as usize) }
  }
}

impl ArrayBuffer {
  pub fn new(size: usize) -> Self {
    ArrayBuffer {
      array: vec![Word::NULL; size].into_boxed_slice(),
    }
  }
}
