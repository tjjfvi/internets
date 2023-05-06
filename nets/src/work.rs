mod pool;
mod steal;
pub use pool::*;
pub use steal::*;

use crate::*;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub struct ActivePair(pub(super) Word, pub(super) Word);

pub trait Work: Debug {
  fn add(&mut self, pair: ActivePair);
  fn take(&mut self) -> Option<ActivePair>;
}

impl Work for Vec<ActivePair> {
  #[inline(always)]
  fn add(&mut self, pair: ActivePair) {
    self.push(pair);
  }
  #[inline(always)]
  fn take(&mut self) -> Option<ActivePair> {
    self.pop()
  }
}
