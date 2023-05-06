use crate::*;
use std::sync::atomic::Ordering;

#[derive(Debug)]
pub struct SplitAlloc<'a, B: Buffer> {
  buffer: &'a B,
  alloc: AtomicAddr,
}

impl<'a, B: Buffer> DelegateBuffer for SplitAlloc<'a, B> {
  type Buffer<'b> = B where Self: 'b;
  #[inline(always)]
  fn buffer(&self) -> &Self::Buffer<'_> {
    &self.buffer
  }
}

impl<'a, B: Buffer> DelegateBuffer for &'a SplitAlloc<'a, B> {
  type Buffer<'b> = B where Self: 'b;
  #[inline(always)]
  fn buffer(&self) -> &Self::Buffer<'_> {
    &self.buffer
  }
}

impl<'a, B: Buffer> Alloc for &'a SplitAlloc<'a, B> {
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

impl<'a, B: Buffer> Alloc for SplitAlloc<'a, B> {
  #[inline(always)]
  fn alloc(&mut self, len: Length) -> Addr {
    (&*self).alloc(len)
  }

  #[inline(always)]
  fn free(&mut self, addr: Addr, len: Length) {
    (&*self).free(addr, len)
  }
}

impl<'a, B: Buffer> SplitAlloc<'a, B> {
  pub fn new(buffer: &'a B, count: usize) -> Vec<Self> {
    (0..count)
      .map(|i| {
        let alloc = AtomicAddr::new(
          buffer.origin() + Length::of(((buffer.len().length_words() * i) / count) as u32),
        );
        SplitAlloc { buffer, alloc }
      })
      .collect()
  }
}
