use crate::*;
use std::ops::{Add, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Addr(pub(super) *mut Word);

impl Addr {
  pub const NULL: Addr = Addr(0 as *mut Word);
}

impl Add<Delta> for Addr {
  type Output = Addr;
  #[inline(always)]
  fn add(self, delta: Delta) -> Self::Output {
    Addr(((self.0 as isize) + (delta.offset_bytes as isize)) as *mut Word)
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
