use crate::*;
use std::{
  ops::{Add, Sub},
  sync::atomic::{AtomicIsize, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Addr(pub(super) *const AtomicWord);

impl Addr {
  pub const NULL: Addr = Addr(0 as *const AtomicWord);
  #[inline(always)]
  pub fn is_null(&self) -> bool {
    self.0 as usize == 0
  }
}

impl Add<Delta> for Addr {
  type Output = Addr;
  #[inline(always)]
  fn add(self, delta: Delta) -> Self::Output {
    Addr((self.0 as *const u8).wrapping_offset(delta.offset_bytes as isize) as *const AtomicWord)
  }
}

impl Add<Length> for Addr {
  type Output = Addr;
  #[inline(always)]
  fn add(self, len: Length) -> Self::Output {
    Addr((self.0 as *const u8).wrapping_offset(len.length_bytes as isize) as *const AtomicWord)
  }
}

impl Sub<Addr> for Addr {
  type Output = Delta;
  #[inline(always)]
  fn sub(self, base: Addr) -> Self::Output {
    Delta {
      offset_bytes: ((self.0 as isize) - (base.0 as isize)) as i32,
    }
  }
}

#[derive(Debug)]
pub struct AtomicAddr(pub(super) AtomicIsize);

impl AtomicAddr {
  pub fn new(value: Addr) -> Self {
    AtomicAddr(AtomicIsize::new(value.0 as isize))
  }
  #[inline(always)]
  pub fn fetch_add(&self, len: Length, order: Ordering) -> Addr {
    Addr(self.0.fetch_add(len.length_bytes as isize, order) as *const AtomicWord)
  }
}
