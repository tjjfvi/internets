mod array;
pub use array::*;

use crate::*;
use std::{fmt::Debug, ops::Range};

pub trait Buffer: Debug {
  fn buffer_bounds(&self) -> Range<Addr>;
  fn assert_valid(&self, addr: Addr, len: Length);

  fn word(&self, addr: Addr) -> Word;
  fn read_payload<P>(&self, addr: Addr) -> P;

  fn origin(&self) -> Addr;
  fn len(&self) -> Length;
}

pub trait BufferMut: Buffer {
  fn word_mut(&mut self, addr: Addr) -> &mut Word;
  fn write_payload<P>(&mut self, addr: Addr, payload: P);
  fn slice_mut(&mut self, addr: Addr, len: Length) -> &mut [Word];
}

pub trait DelegateBuffer: Debug {
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
  #[inline(always)]
  fn buffer_bounds(&self) -> Range<Addr> {
    self.delegatee_buffer().buffer_bounds()
  }
  #[inline(always)]
  fn assert_valid(&self, addr: Addr, len: Length) {
    self.delegatee_buffer().assert_valid(addr, len)
  }
  #[inline(always)]
  fn word(&self, addr: Addr) -> Word {
    self.delegatee_buffer().word(addr)
  }
  #[inline(always)]
  fn read_payload<P>(&self, addr: Addr) -> P {
    self.delegatee_buffer().read_payload(addr)
  }
  #[inline(always)]
  fn origin(&self) -> Addr {
    self.delegatee_buffer().origin()
  }
  #[inline(always)]
  fn len(&self) -> Length {
    self.delegatee_buffer().len()
  }
}

impl<T: DelegateBuffer + DelegateBufferMut> BufferMut for T
where
  T::Buffer: BufferMut,
{
  #[inline(always)]
  fn word_mut(&mut self, addr: Addr) -> &mut Word {
    self.delegatee_buffer_mut().word_mut(addr)
  }
  #[inline(always)]
  fn write_payload<P>(&mut self, addr: Addr, value: P) {
    self.delegatee_buffer_mut().write_payload(addr, value)
  }
  #[inline(always)]
  fn slice_mut(&mut self, addr: Addr, len: Length) -> &mut [Word] {
    self.delegatee_buffer_mut().slice_mut(addr, len)
  }
}
