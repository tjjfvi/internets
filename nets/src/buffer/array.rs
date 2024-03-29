use crate::*;
use std::{
  fmt::Debug,
  ops::{Deref, DerefMut, Range},
  ptr::{read_unaligned, write_unaligned},
};

pub struct ArrayBuffer<T> {
  pub array: T,
}

impl<T: Deref<Target = [Word]>> Debug for ArrayBuffer<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut st = f.debug_map();
    for (i, val) in self.array.iter().enumerate() {
      st.entry(&i, val);
    }
    st.finish()
  }
}

impl<T: Deref<Target = [Word]>> Buffer for ArrayBuffer<T> {
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
    self.assert_valid(addr, Length::of_payload::<P>());
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

impl<T: DerefMut<Target = [Word]>> BufferMut for ArrayBuffer<T> {
  #[inline(always)]
  fn word_mut(&mut self, addr: Addr) -> &mut Word {
    self.assert_valid(addr, Length::of(1));
    unsafe { &mut *addr.0 }
  }

  #[inline(always)]
  fn write_payload<P>(&mut self, addr: Addr, value: P) {
    self.assert_valid(addr, Length::of_payload::<P>());
    unsafe { write_unaligned(addr.0 as *mut P, value) }
  }

  #[inline(always)]
  fn slice_mut(&mut self, addr: Addr, len: Length) -> &mut [Word] {
    unsafe { std::slice::from_raw_parts_mut(addr.0, len.length_words()) }
  }
}

impl ArrayBuffer<Box<[Word]>> {
  pub fn new(size: usize) -> Self {
    ArrayBuffer {
      array: vec![Word::NULL; size].into_boxed_slice(),
    }
  }
}

impl<T: Deref<Target = [Word]>> ArrayBuffer<T> {
  pub fn as_ref(&self) -> ArrayBuffer<&[Word]> {
    ArrayBuffer {
      array: &*self.array,
    }
  }
}

impl<T: DerefMut<Target = [Word]>> ArrayBuffer<T> {
  pub fn as_mut(&mut self) -> ArrayBuffer<&mut [Word]> {
    ArrayBuffer {
      array: &mut *self.array,
    }
  }
}
