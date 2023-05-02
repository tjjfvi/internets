use crate::*;

#[derive(Debug)]
pub struct LinkAlloc<B: BufferMut> {
  pub buffer: B,
  allocs: Vec<Addr>,
  pub end: Addr,
}

impl<B: BufferMut> DelegateBuffer for LinkAlloc<B> {
  type Buffer = B;
  #[inline(always)]
  fn delegatee_buffer(&self) -> &Self::Buffer {
    &self.buffer
  }
}

impl<B: BufferMut> DelegateBufferMut for LinkAlloc<B> {
  #[inline(always)]
  fn delegatee_buffer_mut(&mut self) -> &mut Self::Buffer {
    &mut self.buffer
  }
}

impl<B: BufferMut> Alloc for LinkAlloc<B> {
  #[inline(always)]
  fn alloc(&mut self, len: Length) -> Addr {
    let alloc = self.get_alloc(len);
    if alloc.is_null() {
      let addr = self.end;
      self.end = self.end + len;
      if self.end > self.buffer_bounds().end {
        oom!();
      }
      addr
    } else {
      let addr = alloc;
      *self.get_alloc_mut(len) = self.read_payload(addr);
      addr
    }
  }

  #[inline(always)]
  fn free(&mut self, addr: Addr, len: Length) {
    debug_assert!(len >= Length::of(2));
    self.assert_valid(addr, len);
    if cfg!(debug_assertions) {
      self.slice_mut(addr, len).fill(Word::NULL);
    }
    let alloc = self.get_alloc(len);
    self.write_payload(addr, alloc);
    *self.get_alloc_mut(len) = addr;
  }
}

impl<B: BufferMut> LinkAlloc<B> {
  pub fn new(buffer: B) -> Self {
    safe! { assert!(buffer.len() > Length::of(0)) };
    let alloc_addr = buffer.origin();
    LinkAlloc {
      buffer,
      allocs: vec![Addr::NULL; 256],
      end: alloc_addr,
    }
  }

  #[inline(always)]
  fn get_alloc(&self, len: Length) -> Addr {
    if cfg!(feature = "unsafe") {
      *unsafe { self.allocs.get_unchecked(len.length_words()) }
    } else {
      self.allocs[len.length_words()]
    }
  }

  #[inline(always)]
  fn get_alloc_mut(&mut self, len: Length) -> &mut Addr {
    if cfg!(feature = "unsafe") {
      unsafe { self.allocs.get_unchecked_mut(len.length_words()) }
    } else {
      &mut self.allocs[len.length_words()]
    }
  }
}
