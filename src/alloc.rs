mod bump;
mod ring;

use std::fmt::Debug;

pub use bump::*;
pub use ring::*;

use crate::*;

pub trait Alloc: BufferMut + Debug {
  fn alloc(&mut self, data: &[Word]) -> Addr;
  fn free(&mut self, addr: Addr, len: Length);
}

pub trait DelegateAlloc: Debug {
  type Alloc: Alloc;
  fn delegatee_alloc(&self) -> &Self::Alloc;
  fn delegatee_alloc_mut(&mut self) -> &mut Self::Alloc;
}

impl<T: DelegateAlloc> DelegateBuffer for T {
  type Buffer = T::Alloc;
  #[inline(always)]
  fn delegatee_buffer(&self) -> &Self::Buffer {
    self.delegatee_alloc()
  }
}

impl<T: DelegateAlloc> DelegateBufferMut for T {
  #[inline(always)]
  fn delegatee_buffer_mut(&mut self) -> &mut Self::Buffer {
    self.delegatee_alloc_mut()
  }
}

impl<T: DelegateAlloc> Alloc for T {
  #[inline(always)]
  fn alloc(&mut self, data: &[Word]) -> Addr {
    self.delegatee_alloc_mut().alloc(data)
  }
  #[inline(always)]
  fn free(&mut self, addr: Addr, len: Length) {
    self.delegatee_alloc_mut().free(addr, len)
  }
}
