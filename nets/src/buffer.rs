mod array;
pub use array::*;

use crate::*;
use std::{fmt::Debug, ops::Range};

pub trait Buffer: Debug {
  fn buffer_bounds(&self) -> Range<Addr>;
  fn assert_valid(&self, addr: Addr, len: Length);

  fn read_word(&self, addr: Addr) -> Word;
  fn word(&self, addr: Addr) -> &AtomicWord;

  fn read_payload<P>(&self, addr: Addr) -> P;
  fn write_payload<P>(&self, addr: Addr, payload: P);

  fn origin(&self) -> Addr;
  fn len(&self) -> Length;

  fn write_slice(&self, addr: Addr, len: Length, data: &[Word]);
}

pub trait DelegateBuffer: Debug {
  type Buffer: Buffer;
  fn buffer(&self) -> &Self::Buffer;
  fn buffer_mut(&mut self) -> &mut Self::Buffer;
}

impl<T: DelegateBuffer> Buffer for T {
  #[inline(always)]
  fn buffer_bounds(&self) -> Range<Addr> {
    self.buffer().buffer_bounds()
  }
  #[inline(always)]
  fn assert_valid(&self, addr: Addr, len: Length) {
    self.buffer().assert_valid(addr, len)
  }
  #[inline(always)]
  fn read_word(&self, addr: Addr) -> Word {
    self.buffer().read_word(addr)
  }
  #[inline(always)]
  fn read_payload<P>(&self, addr: Addr) -> P {
    self.buffer().read_payload(addr)
  }
  #[inline(always)]
  fn word(&self, addr: Addr) -> &AtomicWord {
    self.buffer().word(addr)
  }
  #[inline(always)]
  fn origin(&self) -> Addr {
    self.buffer().origin()
  }
  #[inline(always)]
  fn len(&self) -> Length {
    self.buffer().len()
  }
  #[inline(always)]
  fn write_payload<P>(&self, addr: Addr, value: P) {
    self.buffer().write_payload(addr, value)
  }
  #[inline(always)]
  fn write_slice(&self, addr: Addr, len: Length, data: &[Word]) {
    self.buffer().write_slice(addr, len, data)
  }
}
