use crate::*;

#[derive(Debug)]
pub struct BumpAlloc<B: Buffer> {
  buffer: B,
  alloc: Addr,
}

impl<B: Buffer> DelegateBuffer for BumpAlloc<B> {
  type Buffer = B;
  #[inline(always)]
  fn buffer(&self) -> &Self::Buffer {
    &self.buffer
  }
  #[inline(always)]
  fn buffer_mut(&mut self) -> &mut Self::Buffer {
    &mut self.buffer
  }
}

impl<B: Buffer> Alloc for BumpAlloc<B> {
  #[inline(always)]
  fn alloc(&mut self, len: Length) -> Addr {
    let addr = self.alloc;
    self.alloc = addr + len;
    if self.alloc > self.buffer_bounds().end {
      oom!();
    }
    addr
  }

  #[inline(always)]
  fn free(&mut self, addr: Addr, len: Length) {
    if cfg!(debug_assertions) {
      self.slice_mut(addr, len).fill(Word::NULL)
    }
  }
}

impl<B: Buffer> BumpAlloc<B> {
  pub fn new(buffer: B) -> Self {
    let alloc = buffer.origin();
    BumpAlloc { buffer, alloc }
  }
}
