use crate::*;

#[derive(Debug)]
pub struct BumpAlloc<B: BufferMut> {
  buffer: B,
  alloc: Addr,
}

impl<B: BufferMut> DelegateBuffer for BumpAlloc<B> {
  type Buffer = B;
  #[inline(always)]
  fn delegatee_buffer(&self) -> &Self::Buffer {
    &self.buffer
  }
}

impl<B: BufferMut> DelegateBufferMut for BumpAlloc<B> {
  #[inline(always)]
  fn delegatee_buffer_mut(&mut self) -> &mut Self::Buffer {
    &mut self.buffer
  }
}

impl<B: BufferMut> Alloc for BumpAlloc<B> {
  #[inline(always)]
  fn alloc(&mut self, data: &[Word]) -> Addr {
    let len = Length::of(data.len() as u32);
    let addr = self.alloc;
    self.alloc = addr + len;
    self.slice_mut(addr, len).copy_from_slice(data);
    addr
  }

  #[inline(always)]
  fn free(&mut self, addr: Addr, len: Length) {
    if cfg!(debug_assertions) {
      self.slice_mut(addr, len).fill(Word::NULL)
    }
  }
}

impl<B: BufferMut> BumpAlloc<B> {
  pub fn new(buffer: B) -> Self {
    let alloc = buffer.origin();
    BumpAlloc { buffer, alloc }
  }
}
