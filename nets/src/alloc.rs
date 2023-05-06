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
  type Alloc<'a>: Alloc + 'a
  where
    Self: 'a;
  fn alloc<'a>(&'a self) -> &'a Self::Alloc<'a>;
  fn alloc_mut(&mut self) -> &mut Self::Alloc<'_>;
}

impl<T: DelegateAlloc> DelegateBuffer for T {
  type Buffer<'a> = T::Alloc<'a>
  where
    Self: 'a;
  #[inline(always)]
  fn buffer(&self) -> &Self::Buffer<'_> {
    self.alloc()
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
