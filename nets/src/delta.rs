use std::fmt::Debug;

#[derive(Clone, Copy)]
pub struct Delta {
  pub(super) offset_bytes: i32,
}

impl Delta {
  #[inline(always)]
  pub const fn of(delta: i32) -> Delta {
    Delta {
      offset_bytes: delta << 2,
    }
  }
  #[inline(always)]
  pub const fn offset_words(&self) -> i32 {
    self.offset_bytes >> 2
  }
}

impl Debug for Delta {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Delta::of({:?})", self.offset_words())
  }
}
