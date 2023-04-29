mod array;
pub use array::*;

use crate::*;
use std::ops::Range;

pub trait Buffer {
  fn buffer_bounds(&self) -> Range<Addr>;
  fn assert_valid(&self, addr: Addr, width: usize);

  fn word(&self, addr: Addr) -> Word;

  fn origin(&self) -> Addr;
  fn len(&self) -> Delta;
}

pub trait BufferMut: Buffer {
  fn word_mut(&mut self, addr: Addr) -> &mut Word;

  fn slice_mut(&mut self, addr: Addr, len: Delta) -> &mut [Word];
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
  fn assert_valid(&self, addr: Addr, width: usize) {
    self.delegatee_buffer().assert_valid(addr, width)
  }
  fn word(&self, addr: Addr) -> Word {
    self.delegatee_buffer().word(addr)
  }
  fn origin(&self) -> Addr {
    self.delegatee_buffer().origin()
  }
  fn len(&self) -> Delta {
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

  fn slice_mut(&mut self, addr: Addr, len: Delta) -> &mut [Word] {
    self.delegatee_buffer_mut().slice_mut(addr, len)
  }
}
