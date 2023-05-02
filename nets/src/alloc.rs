mod bump;
mod link;
mod ring;

use std::fmt::Debug;

pub use bump::*;
pub use link::*;
pub use ring::*;

use crate::*;

pub trait Alloc: BufferMut + Debug {
  fn alloc(&mut self, len: Length) -> Addr;
  #[inline(always)]
  fn alloc_write(&mut self, data: &[Word]) -> Addr {
    let len = Length::of(data.len() as u32);
    let addr = self.alloc(len);
    self.slice_mut(addr, len).copy_from_slice(data);
    addr
  }
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
  fn alloc(&mut self, len: Length) -> Addr {
    self.delegatee_alloc_mut().alloc(len)
  }
  #[inline(always)]
  fn alloc_write(&mut self, data: &[Word]) -> Addr {
    self.delegatee_alloc_mut().alloc_write(data)
  }
  #[inline(always)]
  fn free(&mut self, addr: Addr, len: Length) {
    self.delegatee_alloc_mut().free(addr, len)
  }
}
