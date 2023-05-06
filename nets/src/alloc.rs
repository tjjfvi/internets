mod bump;
mod link;
mod ring;

use std::fmt::Debug;

pub use bump::*;
pub use link::*;
pub use ring::*;

use crate::*;

pub trait Alloc: Buffer + Debug {
  fn alloc(&mut self, len: Length) -> Addr;
  #[inline(always)]
  fn alloc_write(&mut self, data: &[Word]) -> Addr {
    let len = Length::of(data.len() as u32);
    let addr = self.alloc(len);
    self.write_slice(addr, len, data);
    addr
  }
  fn free(&mut self, addr: Addr, len: Length);
}

pub trait DelegateAlloc: Debug {
  type Alloc: Alloc;
  fn alloc(&self) -> &Self::Alloc;
  fn alloc_mut(&mut self) -> &mut Self::Alloc;
}

impl<T: DelegateAlloc> DelegateBuffer for T {
  type Buffer = T::Alloc;
  #[inline(always)]
  fn buffer(&self) -> &Self::Buffer {
    self.alloc()
  }
  #[inline(always)]
  fn buffer_mut(&mut self) -> &mut Self::Buffer {
    self.alloc_mut()
  }
}

impl<T: DelegateAlloc> Alloc for T {
  #[inline(always)]
  fn alloc(&mut self, len: Length) -> Addr {
    self.alloc_mut().alloc(len)
  }
  #[inline(always)]
  fn alloc_write(&mut self, data: &[Word]) -> Addr {
    self.alloc_mut().alloc_write(data)
  }
  #[inline(always)]
  fn free(&mut self, addr: Addr, len: Length) {
    self.alloc_mut().free(addr, len)
  }
}
