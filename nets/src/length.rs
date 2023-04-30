use std::{
  fmt::Debug,
  ops::{Add, Sub},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Length {
  pub(super) length_bytes: u32,
}

impl Length {
  #[inline(always)]
  pub const fn of(delta: u32) -> Self {
    Length {
      length_bytes: delta << 2,
    }
  }
  #[inline(always)]
  pub const fn length_words(&self) -> usize {
    (self.length_bytes >> 2) as usize
  }
  #[inline(always)]
  pub const fn non_zero(&self) -> bool {
    self.length_bytes != 0
  }
  #[inline(always)]
  pub const fn of_payload<P>() -> Length {
    Length::of(((std::mem::size_of::<P>() + 1) >> 2) as u32)
  }
  #[inline(always)]
  pub const fn add(self, other: Length) -> Length {
    Length {
      length_bytes: self.length_bytes + other.length_bytes,
    }
  }
}

impl Debug for Length {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Delta::of({:?})", self.length_words())
  }
}

impl Add<Length> for Length {
  type Output = Length;
  #[inline(always)]
  fn add(self, length: Length) -> Self::Output {
    Length::add(self, length)
  }
}

impl Sub<Length> for Length {
  type Output = Length;
  #[inline(always)]
  fn sub(self, length: Length) -> Self::Output {
    Length {
      length_bytes: self.length_bytes - length.length_bytes,
    }
  }
}
