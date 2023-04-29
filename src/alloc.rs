mod bump;
mod ring;

pub use bump::*;
pub use ring::*;

use crate::*;

pub trait Alloc: BufferMut {
  fn alloc(&mut self, data: &[Word]) -> Addr;
  fn free(&mut self, addr: Addr, len: Delta);
}

pub trait DelegateAlloc {
  type Alloc: Alloc;
  fn delegatee_alloc(&self) -> &Self::Alloc;
  fn delegatee_alloc_mut(&mut self) -> &mut Self::Alloc;
}

impl<T: DelegateAlloc> DelegateBuffer for T {
  type Buffer = T::Alloc;
  fn delegatee_buffer(&self) -> &Self::Buffer {
    self.delegatee_alloc()
  }
}

impl<T: DelegateAlloc> DelegateBufferMut for T {
  fn delegatee_buffer_mut(&mut self) -> &mut Self::Buffer {
    self.delegatee_alloc_mut()
  }
}

impl<T: DelegateAlloc> Alloc for T {
  fn alloc(&mut self, data: &[Word]) -> Addr {
    self.delegatee_alloc_mut().alloc(data)
  }
  fn free(&mut self, addr: Addr, len: Delta) {
    self.delegatee_alloc_mut().free(addr, len)
  }
}
