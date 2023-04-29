use crate::*;

pub struct BumpAlloc<B: BufferMut> {
  buffer: B,
  alloc: Addr,
}

impl<B: BufferMut> DelegateBuffer for BumpAlloc<B> {
  type Buffer = B;
  fn delegatee_buffer(&self) -> &Self::Buffer {
    &self.buffer
  }
}

impl<B: BufferMut> DelegateBufferMut for BumpAlloc<B> {
  fn delegatee_buffer_mut(&mut self) -> &mut Self::Buffer {
    &mut self.buffer
  }
}

impl<B: BufferMut> Alloc for BumpAlloc<B> {
  fn alloc(&mut self, data: &[Word]) -> Addr {
    let len = Delta::of(data.len() as i32);
    let addr = self.alloc;
    self.alloc = addr + len;
    self.slice_mut(addr, len).copy_from_slice(data);
    addr
  }

  fn free(&mut self, _: Addr, _: Delta) {}
}

impl<B: BufferMut> BumpAlloc<B> {
  pub fn new(buffer: B) -> Self {
    let alloc = buffer.origin();
    BumpAlloc { buffer, alloc }
  }
}
