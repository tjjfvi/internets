use crate::*;
use std::{
  fmt::Debug,
  ops::{Add, Sub},
};

#[derive(Clone, Copy)]
pub struct Delta {
  pub(super) offset_bytes: i32,
}

impl Delta {
  #[inline(always)]
  pub const fn of(delta: i32) -> Delta {
    Delta {
      offset_bytes: delta * (WORD_SIZE as i32),
    }
  }
}

impl Debug for Delta {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Delta::of({:?})", self.offset_bytes / 4)
  }
}

impl Add<Delta> for Delta {
  type Output = Delta;
  #[inline(always)]
  fn add(self, delta: Delta) -> Self::Output {
    Delta {
      offset_bytes: self.offset_bytes + delta.offset_bytes,
    }
  }
}

impl Sub<Delta> for Delta {
  type Output = Delta;
  #[inline(always)]
  fn sub(self, delta: Delta) -> Self::Output {
    Delta {
      offset_bytes: self.offset_bytes - delta.offset_bytes,
    }
  }
}
