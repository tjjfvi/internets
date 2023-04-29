mod array;
pub use array::*;

use crate::*;
use std::ops::Range;

pub trait Buffer {
  fn buffer_bounds(&self) -> Range<Addr>;
  fn assert_valid(&self, addr: Addr, len: Length);

  fn word(&self, addr: Addr) -> Word;

  fn origin(&self) -> Addr;
  fn len(&self) -> Length;
}

pub trait BufferMut: Buffer {
  fn word_mut(&mut self, addr: Addr) -> &mut Word;

  fn slice_mut(&mut self, addr: Addr, len: Length) -> &mut [Word];
}

pub trait DelegateBuffer {
  type Buffer: Buffer;
  fn delegatee_buffer(&self) -> &Self::Buffer;
}

pub trait DelegateBufferMut: DelegateBuffer
where
  Self::Buffer: BufferMut,
{
  fn delegatee_buffer_mut(&mut self) -> &mut Self::Buffer;
}

impl<T: DelegateBuffer> Buffer for T {
  fn buffer_bounds(&self) -> Range<Addr> {
    self.delegatee_buffer().buffer_bounds()
  }
  fn assert_valid(&self, addr: Addr, len: Length) {
    self.delegatee_buffer().assert_valid(addr, len)
  }
  fn word(&self, addr: Addr) -> Word {
    self.delegatee_buffer().word(addr)
  }
  fn origin(&self) -> Addr {
    self.delegatee_buffer().origin()
  }
  fn len(&self) -> Length {
    self.delegatee_buffer().len()
  }
}

impl<T: DelegateBuffer + DelegateBufferMut> BufferMut for T
where
  T::Buffer: BufferMut,
{
  fn word_mut(&mut self, addr: Addr) -> &mut Word {
    self.delegatee_buffer_mut().word_mut(addr)
  }

  fn slice_mut(&mut self, addr: Addr, len: Length) -> &mut [Word] {
    self.delegatee_buffer_mut().slice_mut(addr, len)
  }
}
