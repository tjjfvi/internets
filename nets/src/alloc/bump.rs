use crate::*;
use std::sync::atomic::Ordering;

#[derive(Debug)]
pub struct BumpAlloc<B: Buffer> {
  buffer: B,
  alloc: AtomicAddr,
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
    let addr = self.alloc.fetch_add(len, Ordering::Relaxed);
    if (addr + len) > self.buffer_bounds().end {
      oom!();
    }
    addr
  }

  #[inline(always)]
  fn free(&mut self, addr: Addr, len: Length) {
    if cfg!(debug_assertions) {
      self.write_slice(addr, len, &vec![Word::NULL; len.length_words()][..])
    }
  }
}

impl<B: Buffer> BumpAlloc<B> {
  pub fn new(buffer: B) -> Self {
    let alloc = AtomicAddr::new(buffer.origin());
    BumpAlloc { buffer, alloc }
  }
}
